//! Tauri commands for harmony engine control.
//!
//! Get/set key, mode, scale mode, octave mode, voice leading, interchange,
//! and voice position.

use serde::Serialize;
use tauri::State;

use contrapunk::harmony::{
    CounterpointSpecies, CounterpointStrictness, HarmonyMode, Key, OctaveMode, ScaleMode,
    VoiceLeadingStyle,
};

use crate::state::AppState;

/// Serializable snapshot of the harmony engine state.
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
    })
}

/// Sets the musical key.
#[tauri::command]
pub fn set_key(key: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_key(&key)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_key(parsed);
    Ok(())
}

/// Sets the harmony mode.
#[tauri::command]
pub fn set_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_harmony_mode(&mode)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_mode(parsed);
    Ok(())
}

/// Sets the scale mode.
#[tauri::command]
pub fn set_scale_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_scale_mode(&mode)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_scale_mode(parsed);
    Ok(())
}

/// Sets the octave mode.
#[tauri::command]
pub fn set_octave_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_octave_mode(&mode)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_octave_mode(parsed);
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
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_voice_leading_enabled(enabled);
    engine.set_voice_leading_style(parsed_style);
    Ok(())
}

/// Configures modal interchange.
#[tauri::command]
pub fn set_interchange(enabled: bool, range: u8, state: State<AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_interchange_enabled(enabled);
    engine.set_borrowing_range(range);
    Ok(())
}

/// Sets the number of output voices (1 = melody only, 2+ = melody + harmonies).
#[tauri::command]
pub fn set_voice_count(count: usize, state: State<AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_voice_count(count);
    Ok(())
}

/// Sets the voice position (which voice slot the user input occupies).
#[tauri::command]
pub fn set_voice_position(position: usize, state: State<AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_voice_position(position);
    Ok(())
}

/// Enable or disable auto-key detection.
#[tauri::command]
pub fn set_auto_key(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_auto_key(enabled);
    Ok(())
}

/// Sets the counterpoint species (1-4) used by `HarmonyMode::StrictCounterpoint`.
#[tauri::command]
pub fn set_counterpoint_species(species: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_counterpoint_species(&species)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_counterpoint_species(parsed);
    Ok(())
}

/// Sets the counterpoint strictness (Relaxed or Strict) for scoring weights.
#[tauri::command]
pub fn set_counterpoint_strictness(
    strictness: String,
    state: State<AppState>,
) -> Result<(), String> {
    let parsed = parse_counterpoint_strictness(&strictness)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_counterpoint_strictness(parsed);
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

/// Returns the current humanization configuration.
#[tauri::command]
pub fn get_humanize_state(state: State<AppState>) -> Result<serde_json::Value, String> {
    let config = state.humanize_config.lock().map_err(|e| e.to_string())?;
    serde_json::to_value(&*config).map_err(|e| e.to_string())
}

/// Updates humanization configuration from a partial JSON object.
/// Only fields present in the input are changed; others keep their current value.
#[tauri::command]
pub fn set_humanize_config(
    config: serde_json::Value,
    state: State<AppState>,
) -> Result<(), String> {
    let mut current = state.humanize_config.lock().map_err(|e| e.to_string())?;
    // Merge incoming fields into the existing config
    let mut base = serde_json::to_value(&*current).map_err(|e| e.to_string())?;
    if let (Some(base_obj), Some(incoming)) = (base.as_object_mut(), config.as_object()) {
        for (k, v) in incoming {
            base_obj.insert(k.clone(), v.clone());
        }
    }
    *current = serde_json::from_value(base).map_err(|e| e.to_string())?;
    Ok(())
}

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
