//! Tauri commands for note generator control.
//!
//! Set generator mode, enable/disable, and configure selected notes.

use tauri::State;

use contrapunk::generator::{ArpDirection, ChordType, GeneratorMode};

use crate::state::AppState;

/// Set the generator mode.
///
/// Valid values: `"HeldNotes"`, `"Chord"`, `"ArpeggioUp"`, `"ArpeggioDown"`,
/// `"ArpeggioUpDown"`, `"ScaleRunner"`, `"RandomDiatonic"`.
#[tauri::command]
pub fn set_generator_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_generator_mode(&mode)?;
    let mut gen = state.generator.lock().map_err(|e| e.to_string())?;
    gen.set_mode(parsed);
    Ok(())
}

/// Enable or disable the note generator.
#[tauri::command]
pub fn set_generator_enabled(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let mut gen = state.generator.lock().map_err(|e| e.to_string())?;
    let _events = gen.set_enabled(enabled);
    // NoteOff events from disabling will be handled by the router thread
    // on the next tick cycle.
    Ok(())
}

/// Set the notes the generator should use as source material.
///
/// Accepts an array of MIDI note numbers (0-127).
#[tauri::command]
pub fn set_generator_notes(notes: Vec<u8>, state: State<AppState>) -> Result<(), String> {
    let wmidi_notes: Vec<wmidi::Note> = notes
        .iter()
        .map(|&n| wmidi::Note::from_u8_lossy(n))
        .collect();
    let mut gen = state.generator.lock().map_err(|e| e.to_string())?;
    gen.set_selected_notes(wmidi_notes);
    Ok(())
}

/// Set the chord type for Chord mode.
///
/// Valid values: `"Major"`, `"Minor"`, `"Dim"`, `"Aug"`, `"Maj7"`,
/// `"Min7"`, `"Dom7"`, `"Dim7"`, `"HalfDim7"`.
#[tauri::command]
pub fn set_generator_chord_type(chord_type: String, state: State<AppState>) -> Result<(), String> {
    let ct = parse_chord_type(&chord_type)?;
    let mut gen = state.generator.lock().map_err(|e| e.to_string())?;
    gen.set_mode(GeneratorMode::Chord(ct));
    Ok(())
}

// ============================================================================
// Parsing helpers
// ============================================================================

fn parse_generator_mode(s: &str) -> Result<GeneratorMode, String> {
    match s {
        "HeldNotes" => Ok(GeneratorMode::HeldNotes),
        "Chord" => Ok(GeneratorMode::Chord(ChordType::Major)),
        "ArpeggioUp" => Ok(GeneratorMode::Arpeggio(ArpDirection::Up)),
        "ArpeggioDown" => Ok(GeneratorMode::Arpeggio(ArpDirection::Down)),
        "ArpeggioUpDown" => Ok(GeneratorMode::Arpeggio(ArpDirection::UpDown)),
        "ScaleRunner" => Ok(GeneratorMode::ScaleRunner),
        "RandomDiatonic" => Ok(GeneratorMode::RandomDiatonic),
        other => Err(format!("Unknown generator mode: {}", other)),
    }
}

fn parse_chord_type(s: &str) -> Result<ChordType, String> {
    match s {
        "Major" => Ok(ChordType::Major),
        "Minor" => Ok(ChordType::Minor),
        "Dim" => Ok(ChordType::Dim),
        "Aug" => Ok(ChordType::Aug),
        "Maj7" => Ok(ChordType::Maj7),
        "Min7" => Ok(ChordType::Min7),
        "Dom7" => Ok(ChordType::Dom7),
        "Dim7" => Ok(ChordType::Dim7),
        "HalfDim7" => Ok(ChordType::HalfDim7),
        other => Err(format!("Unknown chord type: {}", other)),
    }
}
