//! Persist MIDI device selections across app restarts.
//!
//! Saves input/output device names to eframe::Storage so they can be
//! restored on next launch. Virtual inputs (Note Generator, Computer Keyboard)
//! are stored as special string constants.

use serde::{Deserialize, Serialize};
use crate::app::{AppState, INPUT_NOTE_GENERATOR, INPUT_COMPUTER_KEYBOARD};

/// Special string constant for persisting the Note Generator virtual input.
const VIRTUAL_NOTE_GENERATOR: &str = "__virtual:note_generator";
/// Special string constant for persisting the Computer Keyboard virtual input.
const VIRTUAL_COMPUTER_KEYBOARD: &str = "__virtual:computer_keyboard";
/// Storage key used in eframe::Storage.
const STORAGE_KEY: &str = "midi_defaults";

/// Saved MIDI device selections.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MidiDefaults {
    /// Saved input device name (or virtual input constant).
    pub input_name: Option<String>,
    /// Saved output device names per slot (None = unassigned).
    pub output_names: Vec<Option<String>>,
}

/// Saves MIDI defaults to eframe storage as JSON.
#[cfg(feature = "gui")]
pub fn save_midi_defaults(storage: &mut dyn eframe::Storage, defaults: &MidiDefaults) {
    if let Ok(json) = serde_json::to_string(defaults) {
        storage.set_string(STORAGE_KEY, json);
    }
}

/// Loads MIDI defaults from eframe storage.
#[cfg(feature = "gui")]
pub fn load_midi_defaults(storage: &dyn eframe::Storage) -> MidiDefaults {
    storage
        .get_string(STORAGE_KEY)
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// Builds a MidiDefaults snapshot from the current app state.
pub fn build_midi_defaults(state: &AppState) -> MidiDefaults {
    let input_name = match state.input_port {
        Some(idx) if idx == INPUT_NOTE_GENERATOR => {
            Some(VIRTUAL_NOTE_GENERATOR.to_string())
        }
        Some(idx) if idx == INPUT_COMPUTER_KEYBOARD => {
            Some(VIRTUAL_COMPUTER_KEYBOARD.to_string())
        }
        Some(idx) => {
            state.available_inputs.iter()
                .find(|(i, _)| *i == idx)
                .map(|(_, name)| name.clone())
        }
        None => None,
    };

    let output_names = state.output_slots.iter().map(|slot| {
        slot.and_then(|idx| {
            state.available_outputs.iter()
                .find(|(i, _)| *i == idx)
                .map(|(_, name)| name.clone())
        })
    }).collect();

    MidiDefaults {
        input_name,
        output_names,
    }
}

/// Applies saved MIDI defaults to the app state by resolving names back to indices.
///
/// Unavailable devices fall back to None (no selection).
pub fn apply_midi_defaults(state: &mut AppState, defaults: &MidiDefaults) {
    // Resolve input
    state.input_port = defaults.input_name.as_deref().and_then(|name| {
        match name {
            VIRTUAL_NOTE_GENERATOR => Some(INPUT_NOTE_GENERATOR),
            VIRTUAL_COMPUTER_KEYBOARD => Some(INPUT_COMPUTER_KEYBOARD),
            _ => state.available_inputs.iter()
                .find(|(_, n)| n == name)
                .map(|(i, _)| *i),
        }
    });

    // Resolve outputs per slot
    for (slot, saved) in state.output_slots.iter_mut().zip(defaults.output_names.iter()) {
        *slot = saved.as_deref().and_then(|name| {
            state.available_outputs.iter()
                .find(|(_, n)| n == name)
                .map(|(i, _)| *i)
        });
    }
}
