//! Tauri commands for harmony engine control.
//!
//! Get/set key, mode, scale mode, octave mode, voice leading, interchange,
//! and voice position.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::State;

use contrapunk::harmony::{
    CounterpointSpecies, CounterpointStrictness, ExplicitIntervalMap, HarmonicLimit, HarmonyMode,
    Key, OctaveMode, ScaleMode, TuningConfig, TuningStyle, VoiceLeadingStyle,
};

use crate::state::AppState;

/// Raise the router-thread panic flag so stuck notes from the previous
/// engine configuration get released via MIDI All-Notes-Off on the next
/// router-loop iteration. Called after any mutation that clears
/// active_notes inside the HarmonyEngine.
fn raise_panic(state: &State<AppState>) {
    state.panic_pending.store(true, Ordering::SeqCst);
}

/// Serializable snapshot of the harmony engine state.
#[derive(Serialize)]
pub struct TuningStateResponse {
    pub tuning_style: TuningStyle,
    pub tuning_depth: f32,
    pub harmonic_limit: HarmonicLimit,
}

#[derive(Serialize)]
pub struct EngineStateResponse {
    pub key: String,
    pub mode: String,
    pub mode_number: u8,
    pub scale_mode: String,
    pub octave_mode: String,
    pub voice_leading_enabled: bool,
    pub voice_leading_style: String,
    pub interchange_enabled: bool,
    pub borrowing_range: u8,
    pub voice_position: usize,
    pub voice_count: usize,
    pub auto_key: bool,
    pub routing_mode: String,
    pub counterpoint_species: String,
    pub counterpoint_strictness: String,
    pub explicit_interval_map: ExplicitIntervalMap,
    #[serde(flatten)]
    pub tuning: TuningStateResponse,
}

/// Returns a snapshot of the current engine configuration.
#[tauri::command]
pub fn get_engine_state(state: State<AppState>) -> Result<EngineStateResponse, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    Ok(EngineStateResponse {
        key: format!("{}", engine.key()),
        mode: format!("{:?}", engine.mode()),
        mode_number: engine.mode().number(),
        scale_mode: format!("{:?}", engine.scale_mode()),
        octave_mode: format!("{:?}", engine.octave_mode()),
        voice_leading_enabled: engine.voice_leading_enabled(),
        voice_leading_style: format!("{:?}", engine.voice_leading_style()),
        interchange_enabled: engine.interchange_enabled(),
        borrowing_range: engine.borrowing_range(),
        voice_position: engine.voice_position(),
        voice_count: engine.voice_count(),
        auto_key: engine.auto_key(),
        routing_mode: format!(
            "{:?}",
            *state.routing_mode.lock().map_err(|e| e.to_string())?
        ),
        counterpoint_species: format!("{:?}", engine.counterpoint_species()),
        counterpoint_strictness: format!("{:?}", engine.counterpoint_strictness()),
        explicit_interval_map: engine.explicit_interval_map().clone(),
        tuning: tuning_state_response(engine.tuning_config()),
    })
}

fn tuning_state_response(config: TuningConfig) -> TuningStateResponse {
    TuningStateResponse {
        tuning_style: config.style,
        tuning_depth: config.depth,
        harmonic_limit: config.harmonic_limit,
    }
}

fn update_tuning(
    state: &State<AppState>,
    update: impl FnOnce(&mut TuningConfig),
) -> Result<(), String> {
    let needs_reharm = {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        let previous = engine.tuning_config();
        let mut next = previous;
        update(&mut next);
        engine
            .set_tuning_config(next)
            .map_err(|error| format!("Invalid tuning configuration: {error:?}"))?;
        previous != next && (previous.style == TuningStyle::Pure || next.style == TuningStyle::Pure)
    };
    if needs_reharm {
        raise_panic(state);
    }
    Ok(())
}

#[tauri::command]
pub fn set_tuning_style(style: String, state: State<AppState>) -> Result<(), String> {
    let style = parse_tuning_style(&style)?;
    update_tuning(&state, |config| config.style = style)
}

#[tauri::command]
pub fn set_tuning_depth(depth: f32, state: State<AppState>) -> Result<(), String> {
    update_tuning(&state, |config| config.depth = depth)
}

#[tauri::command]
pub fn set_harmonic_limit(limit: String, state: State<AppState>) -> Result<(), String> {
    let limit = parse_harmonic_limit(&limit)?;
    update_tuning(&state, |config| config.harmonic_limit = limit)
}

#[tauri::command]
pub fn set_tuning_compare(enabled: bool, state: State<AppState>) -> Result<(), String> {
    state
        .synth_tx
        .set_compare_standard(enabled)
        .map_err(|error| format!("Could not compare tuning: {error}"))
}

/// Returns the engine's current port-map: for each result-index `i`
/// (0 = the user's input/melody, 1..N = harmony voices), the arrangement
/// slot the engine will route that voice through. The VGC reads this so
/// per-voice output labels reflect the engine's actual routing, not the
/// naive `voicePosition`-based guess.
///
/// May be empty if the engine has not processed any notes yet (Idle); in
/// that case the UI should fall back to a config-derived mapping.
#[tauri::command]
pub fn get_last_port_map(state: State<AppState>) -> Result<Vec<usize>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.last_port_map().to_vec())
}

/// Sets the musical key.
#[tauri::command]
pub fn set_key(key: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_key(&key)?;
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_key(parsed);
    }
    raise_panic(&state);
    Ok(())
}

/// Sets the harmony mode.
#[tauri::command]
pub fn set_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_harmony_mode(&mode)?;
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_mode(parsed);
    }
    raise_panic(&state);
    Ok(())
}

/// Sets the scale mode.
#[tauri::command]
pub fn set_scale_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_scale_mode(&mode)?;
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_scale_mode(parsed);
    }
    raise_panic(&state);
    Ok(())
}

/// Sets the octave mode.
#[tauri::command]
pub fn set_octave_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_octave_mode(&mode)?;
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_octave_mode(parsed);
    }
    raise_panic(&state);
    Ok(())
}

/// Configures voice leading.
#[tauri::command]
pub fn set_voice_leading(
    enabled: bool,
    style: String,
    state: State<AppState>,
) -> Result<(), String> {
    let parsed_style = parse_voice_leading_style(&style)?;
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_voice_leading_enabled(enabled);
        engine.set_voice_leading_style(parsed_style);
    }
    raise_panic(&state);
    Ok(())
}

/// Configures modal interchange.
#[tauri::command]
pub fn set_interchange(enabled: bool, range: u8, state: State<AppState>) -> Result<(), String> {
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_interchange_enabled(enabled);
        engine.set_borrowing_range(range);
    }
    raise_panic(&state);
    Ok(())
}

/// Sets the number of output voices (1 = melody only, 2+ = melody + harmonies).
#[tauri::command]
pub fn set_voice_count(count: usize, state: State<AppState>) -> Result<(), String> {
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_voice_count(count);
    }
    raise_panic(&state);
    Ok(())
}

/// Sets the voice position (which voice slot the user input occupies).
#[tauri::command]
pub fn set_voice_position(position: usize, state: State<AppState>) -> Result<(), String> {
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_voice_position(position);
    }
    raise_panic(&state);
    Ok(())
}

/// Enable or disable auto-key detection.
#[tauri::command]
pub fn set_auto_key(enabled: bool, state: State<AppState>) -> Result<(), String> {
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_auto_key(enabled);
    }
    raise_panic(&state);
    Ok(())
}

/// Replaces the source-degree-to-semitone explicit interval map.
#[tauri::command]
pub fn set_explicit_interval_map(
    degree_offsets: [Vec<i8>; 7],
    fallback_offsets: Vec<i8>,
    state: State<AppState>,
) -> Result<(), String> {
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_explicit_interval_map(ExplicitIntervalMap {
            degree_offsets,
            fallback_offsets,
        })?;
    }
    raise_panic(&state);
    Ok(())
}

/// Sets the counterpoint species (1-4) used by `HarmonyMode::StrictCounterpoint`.
#[tauri::command]
pub fn set_counterpoint_species(species: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_counterpoint_species(&species)?;
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_counterpoint_species(parsed);
    }
    raise_panic(&state);
    Ok(())
}

/// Sets the counterpoint strictness (Relaxed or Strict) for scoring weights.
#[tauri::command]
pub fn set_counterpoint_strictness(
    strictness: String,
    state: State<AppState>,
) -> Result<(), String> {
    let parsed = parse_counterpoint_strictness(&strictness)?;
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_counterpoint_strictness(parsed);
    }
    raise_panic(&state);
    Ok(())
}

/// Toggle bass-register suppression (#100). When enabled, input notes
/// below `bass_register_threshold` pass through without producing
/// harmony — for users who play the bass line themselves.
#[tauri::command]
pub fn set_suppress_bass_register(enabled: bool, state: State<AppState>) -> Result<(), String> {
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_suppress_bass_register(enabled);
    }
    raise_panic(&state);
    Ok(())
}

/// Set the MIDI note number at and above which harmony is generated
/// (bass-register threshold). Default 48 (C3). Clamped to 0..=127.
#[tauri::command]
pub fn set_bass_register_threshold(midi: u8, state: State<AppState>) -> Result<(), String> {
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_bass_register_threshold(midi);
    }
    raise_panic(&state);
    Ok(())
}

/// Set the MIDI routing mode (channel-based MPE or port-based).
#[tauri::command]
pub fn set_routing_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let parsed = match mode.as_str() {
        "ChannelBased" | "channel_based" | "channel" | "mpe" => {
            contrapunk::harmony::RoutingMode::ChannelBased
        }
        "PortBased" | "port_based" | "port" | "legacy" => {
            contrapunk::harmony::RoutingMode::PortBased
        }
        other => return Err(format!("Unknown routing mode: {}", other)),
    };
    let mut routing = state.routing_mode.lock().map_err(|e| e.to_string())?;
    *routing = parsed;
    Ok(())
}

/// Continuous octave-spread coefficient applied to Spread / Split modes.
/// Range [0.0, 1.0]; 0 = no displacement, 1 = legacy full-octave behavior.
#[tauri::command]
pub fn set_octave_intensity(amount: f32, state: State<AppState>) -> Result<(), String> {
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_octave_intensity(amount);
    }
    raise_panic(&state);
    Ok(())
}

/// Master enable for the beat-aligned chord-trigger pattern. When false,
/// Set global detune in cents. The router thread reads this atomically each
/// frame and sends MIDI pitch bend to all output ports when the value changes.
#[tauri::command]
pub fn set_detune(cents: i32, state: State<AppState>) -> Result<(), String> {
    state
        .detune_cents
        .store(cents, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

/// Get the current detune value in cents.
#[tauri::command]
pub fn get_detune(state: State<AppState>) -> i32 {
    state
        .detune_cents
        .load(std::sync::atomic::Ordering::Relaxed)
}

// ============================================================================
// Parsing helpers
// ============================================================================

fn parse_tuning_style(s: &str) -> Result<TuningStyle, String> {
    match s {
        "Standard" | "standard" => Ok(TuningStyle::Standard),
        "Pure" | "pure" => Ok(TuningStyle::Pure),
        other => Err(format!("Unknown tuning style: {other}")),
    }
}

fn parse_harmonic_limit(s: &str) -> Result<HarmonicLimit, String> {
    match s {
        "Five" | "five" | "5" => Ok(HarmonicLimit::Five),
        "Seven" | "seven" | "7" => Ok(HarmonicLimit::Seven),
        other => Err(format!("Unknown harmonic limit: {other}")),
    }
}

fn parse_key(s: &str) -> Result<Key, String> {
    match s {
        "C" => Ok(Key::C),
        "Db" | "C#" => Ok(Key::Db),
        "D" => Ok(Key::D),
        "Eb" | "D#" => Ok(Key::Eb),
        "E" => Ok(Key::E),
        "F" => Ok(Key::F),
        "Gb" | "F#" => Ok(Key::Gb),
        "G" => Ok(Key::G),
        "Ab" | "G#" => Ok(Key::Ab),
        "A" => Ok(Key::A),
        "Bb" | "A#" => Ok(Key::Bb),
        "B" => Ok(Key::B),
        other => Err(format!("Unknown key: {}", other)),
    }
}

fn parse_harmony_mode(s: &str) -> Result<HarmonyMode, String> {
    match s {
        "PassThrough" | "pass_through" | "1" => Ok(HarmonyMode::PassThrough),
        "DiatonicThirds" | "diatonic_thirds" | "2" => Ok(HarmonyMode::DiatonicThirds),
        "DiatonicFourths" | "diatonic_fourths" | "3" => Ok(HarmonyMode::DiatonicFourths),
        "RandomBelow" | "random_below" | "4" => Ok(HarmonyMode::RandomBelow),
        "RandomBelowNoSeconds" | "random_below_no_seconds" | "5" => {
            Ok(HarmonyMode::RandomBelowNoSeconds)
        }
        "ContraryMotion" | "contrary_motion" | "6" => Ok(HarmonyMode::ContraryMotion),
        "StrictCounterpoint" | "strict_counterpoint" | "7" => Ok(HarmonyMode::StrictCounterpoint),
        "BarryHarris" | "barry_harris" | "8" => Ok(HarmonyMode::BarryHarris),
        "FunctionalHarmony" | "functional_harmony" | "9" => Ok(HarmonyMode::FunctionalHarmony),
        "BachChorale" | "bach_chorale" | "10" => Ok(HarmonyMode::BachChorale),
        "ExplicitIntervals" | "explicit_intervals" | "11" => Ok(HarmonyMode::ExplicitIntervals),
        other => Err(format!("Unknown harmony mode: {}", other)),
    }
}

fn parse_scale_mode(s: &str) -> Result<ScaleMode, String> {
    // Try serde deserialization (snake_case)
    if let Ok(mode) = serde_json::from_value::<ScaleMode>(serde_json::Value::String(s.to_string()))
    {
        return Ok(mode);
    }
    // Fallback: match common display names
    for mode in ScaleMode::all() {
        let display = format!("{}", mode);
        if display == s || format!("{:?}", mode) == s {
            return Ok(*mode);
        }
    }
    Err(format!("Unknown scale mode: {}", s))
}

fn parse_octave_mode(s: &str) -> Result<OctaveMode, String> {
    match s {
        "None" | "none" => Ok(OctaveMode::None),
        "Spread" | "spread" => Ok(OctaveMode::Spread),
        "BassTrebleSplit" | "bass_treble_split" => Ok(OctaveMode::BassTrebleSplit),
        "Mirror" | "mirror" => Ok(OctaveMode::Mirror),
        other => Err(format!("Unknown octave mode: {}", other)),
    }
}

fn parse_voice_leading_style(s: &str) -> Result<VoiceLeadingStyle, String> {
    match s {
        "Palestrina" | "palestrina" => Ok(VoiceLeadingStyle::Palestrina),
        "BachChorale" | "bach_chorale" | "Bach" | "bach" => Ok(VoiceLeadingStyle::BachChorale),
        "Jazz" | "jazz" => Ok(VoiceLeadingStyle::Jazz),
        "Free" | "free" => Ok(VoiceLeadingStyle::Free),
        other => Err(format!("Unknown voice leading style: {}", other)),
    }
}

fn parse_counterpoint_species(s: &str) -> Result<CounterpointSpecies, String> {
    match s {
        "Species1" | "species1" | "species_1" | "1" => Ok(CounterpointSpecies::Species1),
        "Species2" | "species2" | "species_2" | "2" => Ok(CounterpointSpecies::Species2),
        "Species3" | "species3" | "species_3" | "3" => Ok(CounterpointSpecies::Species3),
        "Species4" | "species4" | "species_4" | "4" => Ok(CounterpointSpecies::Species4),
        other => Err(format!("Unknown counterpoint species: {}", other)),
    }
}

fn parse_counterpoint_strictness(s: &str) -> Result<CounterpointStrictness, String> {
    match s {
        "Relaxed" | "relaxed" => Ok(CounterpointStrictness::Relaxed),
        "Strict" | "strict" => Ok(CounterpointStrictness::Strict),
        other => Err(format!("Unknown counterpoint strictness: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuning_state_serializes_for_tauri() {
        let value = serde_json::to_value(tuning_state_response(TuningConfig {
            style: TuningStyle::Pure,
            depth: 0.75,
            harmonic_limit: HarmonicLimit::Seven,
        }))
        .unwrap();
        assert_eq!(value["tuning_style"], "pure");
        assert_eq!(value["tuning_depth"], 0.75);
        assert_eq!(value["harmonic_limit"], "seven");
    }

    #[test]
    fn tuning_controls_reject_unknown_values() {
        assert!(parse_tuning_style("Color").is_err());
        assert!(parse_harmonic_limit("Eleven").is_err());
    }
}
