//! GUI application for Contrapunk MIDI harmony generator.
//!
//! This module provides the eframe/egui-based GUI interface.

use std::collections::HashSet;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::thread::JoinHandle;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

use anyhow::Result;
use eframe::egui;
use crate::chord::chord_display_with_analysis;
use crate::harmony::ScaleMode;
use crate::generator::{NoteGenerator, GeneratorMode, GeneratorEvent, ChordType, ArpDirection};
use crate::harmony::{Key, HarmonyMode, OctaveMode, HarmonyEngine, VoiceLeadingStyle};
use crate::humanize::HumanizeConfig;
#[cfg(target_arch = "wasm32")]
use crate::humanize::{Humanizer, DelayQueue, Metronome};
use crate::piano::PianoKeyboard;
use crate::preset::{PresetManager, storage as preset_storage};
use crate::theme::ContrapunkTheme;
use crate::theme::widgets::{draw_ornate_frame, draw_scanlines};
use crate::theme::colors::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::midi::ports::{list_input_ports, list_output_ports};
#[cfg(not(target_arch = "wasm32"))]
use crate::router::{spawn_gui_router, GUIRouterState};
#[cfg(target_arch = "wasm32")]
use crate::midi::web;
#[cfg(target_arch = "wasm32")]
use wmidi::{MidiMessage, Note, Channel, Velocity};

/// Converts a MIDI note number to a note name string (e.g., 60 -> "C4").
pub(crate) fn midi_to_name(midi: u8) -> String {
    const NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (midi / 12) as i8 - 1; // MIDI 60 = C4
    let note = NOTES[(midi % 12) as usize];
    format!("{}{}", note, octave)
}

/// Maximum number of output slots available in the GUI.
const MAX_OUTPUT_SLOTS: usize = 8;

/// Application state shared across the GUI.
///
/// Contains all the data needed to display and control the harmony engine.
pub struct AppState {
    /// Current musical key
    pub key: Key,
    /// Current harmony mode
    pub mode: HarmonyMode,
    /// Current octave mode for harmony voices
    pub octave_mode: OctaveMode,
    /// Voice position: which voice slot the user plays (0 = top/soprano, max = bass)
    pub voice_position: usize,
    /// Selected input port index
    pub input_port: Option<usize>,
    /// Selected output port indices (one per slot, None if slot not assigned)
    pub output_slots: Vec<Option<usize>>,
    /// Available input ports (index, name)
    pub available_inputs: Vec<(usize, String)>,
    /// Available output ports (index, name)
    pub available_outputs: Vec<(usize, String)>,
    /// Whether MIDI routing is active
    pub is_running: bool,
    /// Last error message for display
    pub last_error: Option<String>,
}

impl AppState {
    /// Refreshes the available MIDI device lists.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn refresh_devices(&mut self) {
        self.last_error = None;

        match list_input_ports() {
            Ok(inputs) => self.available_inputs = inputs,
            Err(e) => {
                self.last_error = Some(format!("Failed to list input ports: {}", e));
                self.available_inputs.clear();
            }
        }

        match list_output_ports() {
            Ok(outputs) => self.available_outputs = outputs,
            Err(e) => {
                self.last_error = Some(format!("Failed to list output ports: {}", e));
                self.available_outputs.clear();
            }
        }

        // Validate current selections still exist
        if let Some(input_idx) = self.input_port {
            if !self.available_inputs.iter().any(|(i, _)| *i == input_idx) {
                self.input_port = None;
            }
        }

        // Validate output selections
        for slot in &mut self.output_slots {
            if let Some(idx) = *slot {
                if !self.available_outputs.iter().any(|(i, _)| *i == idx) {
                    *slot = None;
                }
            }
        }
    }

    /// Refreshes the available MIDI device lists (WASM).
    ///
    /// Note: On WASM, device refresh is handled via the shared MidiAccess
    /// object. This method is a no-op since the app polls MidiAccess in update().
    #[cfg(target_arch = "wasm32")]
    pub fn refresh_devices(&mut self) {
        // Device enumeration happens in ContrapunkApp::update() via midi_access
        self.last_error = None;
    }

    /// Returns selected output ports as a Vec<usize> (filtering out None slots).
    pub fn selected_output_ports(&self) -> Vec<usize> {
        self.output_slots.iter().filter_map(|s| *s).collect()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            key: Key::C,
            mode: HarmonyMode::PassThrough,
            octave_mode: OctaveMode::None,
            voice_position: MAX_OUTPUT_SLOTS - 1, // Bass (clamped to voice_count-1 when routing starts)
            input_port: None,
            output_slots: vec![None; MAX_OUTPUT_SLOTS],
            available_inputs: Vec::new(),
            available_outputs: Vec::new(),
            is_running: false,
            last_error: None,
        }
    }
}

/// Main application struct for eframe.
pub struct ContrapunkApp {
    pub(crate) state: AppState,
    /// Shared state for router communication
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) router_state: Option<Arc<Mutex<GUIRouterState>>>,
    /// Handle to the router thread
    #[cfg(not(target_arch = "wasm32"))]
    router_handle: Option<JoinHandle<Result<()>>>,
    /// Web MIDI access object (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub(crate) midi_access: Rc<RefCell<Option<web_sys::MidiAccess>>>,
    /// Shared MIDI message queue — input callback pushes, update() drains (WASM only)
    #[cfg(target_arch = "wasm32")]
    midi_queue: Rc<RefCell<Vec<Vec<u8>>>>,
    /// Selected output device IDs for Web MIDI (WASM only)
    #[cfg(target_arch = "wasm32")]
    connected_output_ids: Vec<String>,
    /// Whether Web MIDI initialization has been attempted (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub(crate) midi_initialized: bool,
    /// Harmony engine for WASM frame-based processing
    #[cfg(target_arch = "wasm32")]
    engine: HarmonyEngine,
    /// Active input notes for WASM display
    #[cfg(target_arch = "wasm32")]
    wasm_input_notes: HashSet<u8>,
    /// Active harmony notes for WASM display
    #[cfg(target_arch = "wasm32")]
    wasm_harmony_notes: HashSet<u8>,
    /// Preset manager for built-in and custom presets
    pub(crate) preset_manager: PresetManager,
    /// Deferred start signal (set by Play tab, consumed in update)
    pub(crate) pending_start: bool,
    /// Deferred stop signal (set by Play tab, consumed in update)
    pub(crate) pending_stop: bool,
    /// Whether the theme has been applied (apply only once)
    theme_applied: bool,
    /// Music-reactive activity level (0.0 = silent, 1.0 = active)
    note_activity: f32,
    /// Previous frame note count for detecting new note events
    prev_note_count: usize,
    /// Current animation time (set each frame)
    anim_time: f32,
    /// Whether voice leading is enabled
    pub(crate) voice_leading_enabled: bool,
    /// Current voice leading style
    pub(crate) voice_leading_style: VoiceLeadingStyle,
    /// Current scale mode
    pub(crate) scale_mode: ScaleMode,
    /// Whether modal interchange is enabled
    pub(crate) interchange_enabled: bool,
    /// Borrowing range for modal interchange (1-5)
    pub(crate) borrowing_range: u8,
    /// Local copy of humanization config for GUI editing
    pub(crate) humanize_config: HumanizeConfig,
    /// Humanizer engine for WASM note processing
    #[cfg(target_arch = "wasm32")]
    wasm_humanizer: Humanizer,
    /// Delay queue for WASM humanized note scheduling
    #[cfg(target_arch = "wasm32")]
    wasm_delay_queue: DelayQueue,
    /// Metronome for WASM beat clicks
    #[cfg(target_arch = "wasm32")]
    wasm_metronome: Metronome,
    /// Whether the beat clock has been started (WASM)
    #[cfg(target_arch = "wasm32")]
    wasm_clock_started: bool,
    // --- Craft tab UI state ---
    /// Whether Save As dialog is open
    pub(crate) craft_save_as_open: bool,
    /// Name input for Save As
    pub(crate) craft_save_as_name: String,
    /// Persona input for Save As
    pub(crate) craft_save_as_persona: String,
    /// Genre input for Save As
    pub(crate) craft_save_as_genre: String,
    /// Export JSON text area content
    pub(crate) craft_export_json: String,
    /// Import JSON text area content
    pub(crate) craft_import_json: String,
    /// Whether import section is open
    pub(crate) craft_import_open: bool,
    // --- Generator state ---
    /// Note generator engine
    pub(crate) generator: NoteGenerator,
    /// Whether generator is enabled (UI toggle)
    pub(crate) generator_enabled: bool,
    /// Current generator mode index for UI combo box
    pub(crate) generator_mode_index: usize,
    /// Chord root note index (0=C .. 11=B) for chord mode
    pub(crate) generator_chord_root: usize,
    /// Chord quality index for chord mode
    pub(crate) generator_chord_quality: usize,
    /// Selected notes for piano click-to-select (MIDI note numbers)
    pub(crate) generator_selected_notes: Vec<u8>,
    /// Whether computer keyboard input is enabled
    pub(crate) keyboard_enabled: bool,
    /// Currently held keyboard keys (MIDI note numbers) for proper NoteOff
    pub(crate) keyboard_held_keys: std::collections::HashSet<u8>,
}

/// Sentinel port index for "Note Generator" virtual input.
pub const INPUT_NOTE_GENERATOR: usize = usize::MAX;
/// Sentinel port index for "Computer Keyboard" virtual input.
pub const INPUT_COMPUTER_KEYBOARD: usize = usize::MAX - 1;

impl ContrapunkApp {
    /// Create a new ContrapunkApp instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let midi_access = Rc::new(RefCell::new(None));
        #[cfg(target_arch = "wasm32")]
        let midi_queue = Rc::new(RefCell::new(Vec::new()));

        let mut app = Self {
            state: AppState::default(),
            #[cfg(not(target_arch = "wasm32"))]
            router_state: None,
            #[cfg(not(target_arch = "wasm32"))]
            router_handle: None,
            #[cfg(target_arch = "wasm32")]
            midi_access: midi_access.clone(),
            #[cfg(target_arch = "wasm32")]
            midi_queue: midi_queue.clone(),
            #[cfg(target_arch = "wasm32")]
            connected_output_ids: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            midi_initialized: false,
            #[cfg(target_arch = "wasm32")]
            engine: HarmonyEngine::new(Key::C, HarmonyMode::PassThrough),
            #[cfg(target_arch = "wasm32")]
            wasm_input_notes: HashSet::new(),
            #[cfg(target_arch = "wasm32")]
            wasm_harmony_notes: HashSet::new(),
            preset_manager: PresetManager::new(),
            pending_start: false,
            pending_stop: false,
            theme_applied: false,
            note_activity: 0.0,
            prev_note_count: 0,
            anim_time: 0.0,
            voice_leading_enabled: false,
            voice_leading_style: VoiceLeadingStyle::default(),
            scale_mode: ScaleMode::default(),
            interchange_enabled: false,
            borrowing_range: 3,
            humanize_config: HumanizeConfig::default(),
            #[cfg(target_arch = "wasm32")]
            wasm_humanizer: Humanizer::new(HumanizeConfig::default()),
            #[cfg(target_arch = "wasm32")]
            wasm_delay_queue: DelayQueue::new(),
            #[cfg(target_arch = "wasm32")]
            wasm_metronome: Metronome::new(),
            #[cfg(target_arch = "wasm32")]
            wasm_clock_started: false,
            craft_save_as_open: false,
            craft_save_as_name: String::new(),
            craft_save_as_persona: String::new(),
            craft_save_as_genre: String::new(),
            craft_export_json: String::new(),
            craft_import_json: String::new(),
            craft_import_open: false,
            generator: NoteGenerator::new(),
            generator_enabled: false,
            generator_mode_index: 0,
            generator_chord_root: 0,
            generator_chord_quality: 0,
            generator_selected_notes: Vec::new(),
            keyboard_enabled: false,
            keyboard_held_keys: std::collections::HashSet::new(),
        };

        // Load custom presets from eframe storage
        if let Some(storage) = _cc.storage {
            let custom = preset_storage::load_custom_presets(storage);
            if !custom.is_empty() {
                app.preset_manager.set_custom_presets(custom);
            }
        }

        // Auto-refresh devices on startup
        app.state.refresh_devices();

        // Initialize Web MIDI asynchronously (WASM only)
        #[cfg(target_arch = "wasm32")]
        {
            let access_ref = midi_access.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match web::request_midi_access().await {
                    Ok(access) => {
                        *access_ref.borrow_mut() = Some(access);
                    }
                    Err(_e) => {
                        // Web MIDI not available — will show message in UI
                    }
                }
            });
        }

        app
    }

    /// Returns whether routing is currently active.
    pub(crate) fn is_running(&self) -> bool {
        self.state.is_running
    }

    /// Validates configuration and attempts to start routing.
    #[cfg(not(target_arch = "wasm32"))]
    fn try_start(&mut self, ctx: &egui::Context) {
        self.state.last_error = None;

        // Validate input port is selected
        let input_port = match self.state.input_port {
            Some(idx) => idx,
            None => {
                self.state.last_error = Some("Please select an input device".to_string());
                return;
            }
        };

        // Validate at least one output is selected
        let output_ports = self.state.selected_output_ports();
        if output_ports.is_empty() {
            self.state.last_error = Some("Please select at least one output device".to_string());
            return;
        }

        // Create shared state
        let router_state = Arc::new(Mutex::new(GUIRouterState::default()));
        self.router_state = Some(router_state.clone());

        // Spawn the router
        match spawn_gui_router(
            input_port,
            output_ports,
            self.state.key,
            self.state.mode,
            self.state.octave_mode,
            router_state,
            ctx.clone(),
        ) {
            Ok(handle) => {
                self.router_handle = Some(handle);
                self.state.is_running = true;
            }
            Err(e) => {
                self.state.last_error = Some(format!("Failed to start router: {}", e));
                self.router_state = None;
            }
        }
    }

    /// Validates configuration and attempts to start routing (WASM).
    #[cfg(target_arch = "wasm32")]
    fn try_start(&mut self, _ctx: &egui::Context) {
        self.state.last_error = None;

        // Validate input port is selected
        let input_idx = match self.state.input_port {
            Some(idx) => idx,
            None => {
                self.state.last_error = Some("Please select an input device".to_string());
                return;
            }
        };

        let is_virtual_input = input_idx == INPUT_NOTE_GENERATOR || input_idx == INPUT_COMPUTER_KEYBOARD;

        // Validate at least one output is selected
        let output_ports = self.state.selected_output_ports();
        if output_ports.is_empty() {
            self.state.last_error = Some("Please select at least one output device".to_string());
            return;
        }

        let access = self.midi_access.borrow();
        let access_ref = match access.as_ref() {
            Some(a) => Some(a),
            None if is_virtual_input => None,
            None => {
                self.state.last_error = Some("Web MIDI not available. Use Chrome/Edge with MIDI device.".to_string());
                return;
            }
        };

        // Collect output device IDs
        self.connected_output_ids.clear();
        if let Some(access) = access_ref {
            let outputs = web::list_midi_outputs(access);
            for idx in &output_ports {
                if let Some((id, _)) = outputs.get(*idx) {
                    self.connected_output_ids.push(id.clone());
                }
            }

            // Connect physical input callback (skip for virtual inputs)
            if !is_virtual_input {
                let inputs = web::list_midi_inputs(access);
                let input_id = match inputs.get(input_idx) {
                    Some((id, _)) => id.clone(),
                    None => {
                        self.state.last_error = Some("Selected input device not found".to_string());
                        return;
                    }
                };
                if let Err(_e) = web::connect_input(access, &input_id, self.midi_queue.clone()) {
                    self.state.last_error = Some("Failed to connect to MIDI input".to_string());
                    return;
                }
            }
        }
        drop(access);

        // Configure harmony engine
        self.engine = HarmonyEngine::new(self.state.key, self.state.mode);
        let num_outputs = self.connected_output_ids.len().max(1);
        self.engine.set_voice_count(num_outputs);
        self.engine.set_octave_mode(self.state.octave_mode);
        self.engine.set_voice_position(self.state.voice_position);

        self.wasm_input_notes.clear();
        self.wasm_harmony_notes.clear();
        self.state.is_running = true;
    }

    /// Stops routing.
    #[cfg(not(target_arch = "wasm32"))]
    fn stop(&mut self) {
        // Signal the router thread to stop
        if let Some(ref router_state) = self.router_state {
            if let Ok(mut state) = router_state.lock() {
                state.stop_signal = true;
            }
        }

        // Wait for the router thread to finish
        if let Some(handle) = self.router_handle.take() {
            // Give it a moment to stop gracefully
            let _ = handle.join();
        }

        // Clean up
        self.router_state = None;
        self.state.is_running = false;
    }

    /// Stops routing (WASM).
    #[cfg(target_arch = "wasm32")]
    fn stop(&mut self) {
        self.state.is_running = false;
        self.wasm_input_notes.clear();
        self.wasm_harmony_notes.clear();
        self.midi_queue.borrow_mut().clear();
        self.wasm_humanizer.clock_mut().stop();
        self.wasm_clock_started = false;
    }

    /// Gets the current input/harmony notes from the router state.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn get_router_notes(&self) -> (HashSet<u8>, HashSet<u8>) {
        if let Some(ref router_state) = self.router_state {
            if let Ok(state) = router_state.lock() {
                return (state.input_notes.clone(), state.harmony_notes.clone());
            }
        }
        (HashSet::new(), HashSet::new())
    }

    /// Gets the current input/harmony notes (WASM).
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn get_router_notes(&self) -> (HashSet<u8>, HashSet<u8>) {
        (self.wasm_input_notes.clone(), self.wasm_harmony_notes.clone())
    }

    /// Creates a StylePreset from the current app state.
    pub(crate) fn preset_from_current(&self, name: &str, persona: &str, genre: &str) -> crate::preset::StylePreset {
        crate::preset::StylePreset {
            name: name.to_string(),
            persona: persona.to_string(),
            genre: genre.to_string(),
            harmony_mode: self.state.mode,
            key: self.state.key,
            voice_leading_enabled: self.voice_leading_enabled,
            voice_leading_style: self.voice_leading_style,
            octave_mode: self.state.octave_mode,
            humanize_config: self.humanize_config.clone(),
            scale_mode: self.scale_mode,
            interchange_enabled: self.interchange_enabled,
            borrowing_range: self.borrowing_range,
            is_builtin: false,
        }
    }

    /// Applies a specific preset's settings to app state.
    pub(crate) fn apply_preset(&mut self, preset: &crate::preset::StylePreset) {
        self.state.key = preset.key;
        self.state.mode = preset.harmony_mode;
        self.state.octave_mode = preset.octave_mode;
        self.voice_leading_enabled = preset.voice_leading_enabled;
        self.voice_leading_style = preset.voice_leading_style;
        self.humanize_config = preset.humanize_config.clone();
        self.scale_mode = preset.scale_mode;
        self.interchange_enabled = preset.interchange_enabled;
        self.borrowing_range = preset.borrowing_range;

        // Signal router thread of key/mode/octave changes
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref router_state) = self.router_state {
            if let Ok(mut state_lock) = router_state.lock() {
                state_lock.key = Some(preset.key);
                state_lock.mode = Some(preset.harmony_mode);
                state_lock.octave_mode = Some(preset.octave_mode);
            }
        }
    }

    /// Handle input port selection change: enable/disable virtual inputs, send all_notes_off.
    pub(crate) fn on_input_port_changed(&mut self, _prev: Option<usize>) {
        // Send all notes off for generator
        let off_events = self.generator.all_notes_off();
        // Clear keyboard held keys
        self.keyboard_held_keys.clear();

        match self.state.input_port {
            Some(INPUT_NOTE_GENERATOR) => {
                self.generator_enabled = true;
                self.keyboard_enabled = false;
                let _ = self.generator.set_enabled(true);
            }
            Some(INPUT_COMPUTER_KEYBOARD) => {
                self.generator_enabled = false;
                self.keyboard_enabled = true;
                let _ = self.generator.set_enabled(false);
            }
            _ => {
                // Physical device or None
                self.generator_enabled = false;
                self.keyboard_enabled = false;
                let _ = self.generator.set_enabled(false);
            }
        }

        // Process any pending note-off events through harmony engine (WASM)
        #[cfg(target_arch = "wasm32")]
        {
            for event in off_events {
                if let crate::generator::GeneratorEvent::NoteOff(note) = event {
                    self.handle_wasm_note_off(
                        wmidi::Channel::Ch1,
                        note,
                        wmidi::Velocity::try_from(0u8).unwrap(),
                    );
                }
            }
        }

        // For native, the router thread handles note-off via shared state
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = off_events; // Router thread will pick up enabled=false and handle cleanup
        }
    }

    /// Maps a keyboard key to a MIDI note number using QWERTY piano layout.
    /// Returns None if the key doesn't map to a note.
    pub(crate) fn key_to_midi_note(key: egui::Key) -> Option<u8> {
        match key {
            // Bottom row: C3-B3 (white keys)
            egui::Key::Z => Some(48),  // C3
            egui::Key::S => Some(49),  // C#3
            egui::Key::X => Some(50),  // D3
            egui::Key::D => Some(51),  // D#3
            egui::Key::C => Some(52),  // E3
            egui::Key::V => Some(53),  // F3
            egui::Key::G => Some(54),  // F#3
            egui::Key::B => Some(55),  // G3
            egui::Key::H => Some(56),  // G#3
            egui::Key::N => Some(57),  // A3
            egui::Key::J => Some(58),  // A#3
            egui::Key::M => Some(59),  // B3
            // Top row: C4-B4 (white keys)
            egui::Key::Q => Some(60),  // C4
            egui::Key::Num2 => Some(61), // C#4
            egui::Key::W => Some(62),  // D4
            egui::Key::Num3 => Some(63), // D#4
            egui::Key::E => Some(64),  // E4
            egui::Key::R => Some(65),  // F4
            egui::Key::Num5 => Some(66), // F#4
            egui::Key::T => Some(67),  // G4
            egui::Key::Num6 => Some(68), // G#4
            egui::Key::Y => Some(69),  // A4
            egui::Key::Num7 => Some(70), // A#4
            egui::Key::U => Some(71),  // B4
            egui::Key::I => Some(72),  // C5
            _ => None,
        }
    }

    /// Applies the currently active preset's settings to app state.
    pub(crate) fn apply_active_preset(&mut self) {
        if let Some(preset) = self.preset_manager.active().cloned() {
            self.apply_preset(&preset);
        }
    }
}

/// WASM-specific MIDI processing methods.
#[cfg(target_arch = "wasm32")]
impl ContrapunkApp {
    /// Process a single MIDI message through the harmony engine (frame-based).
    fn process_wasm_midi(&mut self, bytes: &[u8]) {
        let msg = match MidiMessage::try_from(bytes) {
            Ok(m) => m,
            Err(_) => return,
        };

        match msg {
            MidiMessage::NoteOn(channel, note, velocity) => {
                if velocity == Velocity::MIN {
                    self.handle_wasm_note_off(channel, note, velocity);
                } else {
                    self.handle_wasm_note_on(channel, note, velocity);
                }
            }
            MidiMessage::NoteOff(channel, note, velocity) => {
                self.handle_wasm_note_off(channel, note, velocity);
            }
            _ => {
                // Non-note messages: send to first output
                if let Some(id) = self.connected_output_ids.first() {
                    if let Some(ref access) = *self.midi_access.borrow() {
                        let _ = web::send_to_output(access, id, bytes);
                    }
                }
            }
        }
    }

    fn handle_wasm_note_on(&mut self, channel: Channel, note: Note, velocity: Velocity) {
        let notes = self.engine.harmonize_note_on(note);
        let port_map = self.engine.last_port_map();
        let num_outputs = self.connected_output_ids.len();

        // Update display state
        self.wasm_input_notes.insert(note as u8);
        for &n in notes.iter().skip(1) {
            self.wasm_harmony_notes.insert(n as u8);
        }

        let now_ms = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        // Skip metronome-dedicated port for harmony/melody notes
        let metro_reserved = if self.humanize_config.metronome_enabled {
            self.humanize_config.metronome_output_port
        } else {
            None
        };

        // Send to Web MIDI outputs
        let access = self.midi_access.borrow();
        if let Some(ref access) = *access {
            for (i, &n) in notes.iter().enumerate() {
                let port = if i < port_map.len() { port_map[i] } else { i };
                if port >= num_outputs || Some(port) == metro_reserved {
                    continue;
                }

                if i == 0 {
                    // Melody (index 0): pass through unchanged
                    let msg = MidiMessage::NoteOn(channel, n, velocity);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    if msg.copy_to_slice(&mut buf).is_ok() {
                        let _ = web::send_to_output(access, &self.connected_output_ids[port], &buf);
                    }
                } else {
                    // Harmony notes: process through humanizer
                    let hn = self.wasm_humanizer.humanize_note_on(n, channel, velocity, port);
                    if hn.delay_ms == 0 {
                        let msg = MidiMessage::NoteOn(channel, hn.note, hn.velocity);
                        let mut buf = vec![0u8; msg.bytes_size()];
                        if msg.copy_to_slice(&mut buf).is_ok() {
                            let _ = web::send_to_output(access, &self.connected_output_ids[port], &buf);
                        }
                    } else {
                        self.wasm_delay_queue.push(hn, now_ms);
                    }
                }
            }
        }
    }

    fn handle_wasm_note_off(&mut self, channel: Channel, note: Note, velocity: Velocity) {
        let notes = self.engine.harmonize_note_off(note);
        let port_map = self.engine.last_port_map();
        let num_outputs = self.connected_output_ids.len();

        // Update display state
        self.wasm_input_notes.remove(&(note as u8));
        for &n in notes.iter().skip(1) {
            self.wasm_harmony_notes.remove(&(n as u8));
        }

        let now_ms = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        // Skip metronome-dedicated port for harmony/melody notes
        let metro_reserved = if self.humanize_config.metronome_enabled {
            self.humanize_config.metronome_output_port
        } else {
            None
        };

        // Send to Web MIDI outputs
        let access = self.midi_access.borrow();
        if let Some(ref access) = *access {
            for (i, &n) in notes.iter().enumerate() {
                let port = if i < port_map.len() { port_map[i] } else { i };
                if port >= num_outputs || Some(port) == metro_reserved {
                    continue;
                }

                if i == 0 {
                    // Melody (index 0): pass through unchanged
                    let msg = MidiMessage::NoteOff(channel, n, velocity);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    if msg.copy_to_slice(&mut buf).is_ok() {
                        let _ = web::send_to_output(access, &self.connected_output_ids[port], &buf);
                    }
                } else {
                    // Harmony notes: process through humanizer for matching delay
                    let hn = self.wasm_humanizer.humanize_note_off(n, channel, velocity, port);
                    if hn.delay_ms == 0 {
                        let msg = MidiMessage::NoteOff(channel, hn.note, hn.velocity);
                        let mut buf = vec![0u8; msg.bytes_size()];
                        if msg.copy_to_slice(&mut buf).is_ok() {
                            let _ = web::send_to_output(access, &self.connected_output_ids[port], &buf);
                        }
                    } else {
                        self.wasm_delay_queue.push(hn, now_ms);
                    }
                }
            }
        }
    }
}

impl eframe::App for ContrapunkApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        preset_storage::save_custom_presets(storage, self.preset_manager.custom_presets());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply steampunk theme once on first frame
        if !self.theme_applied {
            ContrapunkTheme::apply(ctx);
            self.theme_applied = true;
        }

        // --- Voice position shortcuts (1-4 keys) ---
        {
            let num_outputs = self.state.output_slots.iter().filter(|s| s.is_some()).count().max(1);
            let new_vp = ctx.input(|input| {
                for event in &input.events {
                    if let egui::Event::Key { key, pressed: true, repeat: false, .. } = event {
                        let pos = match key {
                            egui::Key::Num1 => Some(0),
                            egui::Key::Num2 => Some(1),
                            egui::Key::Num3 => Some(2),
                            egui::Key::Num4 => Some(3),
                            _ => None,
                        };
                        if let Some(p) = pos {
                            if p < num_outputs {
                                return Some(p);
                            }
                        }
                    }
                }
                None
            });
            if let Some(vp) = new_vp {
                self.state.voice_position = vp;
            }
        }

        // --- Ambient animations (rendered after panels so they're visible) ---
        // We store animation state here but render AFTER CentralPanel below.
        {
            let time = ctx.input(|i| i.time) as f32;
            self.anim_time = time;

            // Update note activity from current notes
            let (input_n, harmony_n) = self.get_router_notes();
            let current_count = input_n.len() + harmony_n.len();
            if current_count > 0 {
                self.note_activity = 1.0;
            } else {
                self.note_activity *= 0.95;
            }
            self.prev_note_count = current_count;

            // Request repaint at 30fps for ambient animations
            ctx.request_repaint_after(Duration::from_millis(33));
        }

        // Title bar — padded to clear the ornate border overlay
        egui::TopBottomPanel::top("title_bar")
            .frame(egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 12, top: 10, bottom: 4 }))
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("CONTRAPUNK")
                    .color(GOLD).size(18.0).strong());
            });

        // WASM: populate device lists from MidiAccess and process MIDI each frame
        #[cfg(target_arch = "wasm32")]
        {
            // Once MidiAccess is available, populate device lists
            if let Some(ref access) = *self.midi_access.borrow() {
                if !self.midi_initialized {
                    self.midi_initialized = true;
                    let inputs = web::list_midi_inputs(access);
                    self.state.available_inputs = inputs.iter().enumerate()
                        .map(|(i, (_, name))| (i, name.clone()))
                        .collect();
                    let outputs = web::list_midi_outputs(access);
                    self.state.available_outputs = outputs.iter().enumerate()
                        .map(|(i, (_, name))| (i, name.clone()))
                        .collect();
                }
            }

            // Frame-based MIDI processing when running
            if self.state.is_running {
                let now_ms = web_sys::window()
                    .and_then(|w| w.performance())
                    .map(|p| p.now())
                    .unwrap_or(0.0);

                // Sync voice leading state to engine each frame
                if self.engine.voice_leading_enabled() != self.voice_leading_enabled {
                    self.engine.set_voice_leading_enabled(self.voice_leading_enabled);
                }
                if self.engine.voice_leading_style() != self.voice_leading_style {
                    self.engine.set_voice_leading_style(self.voice_leading_style);
                }
                if self.engine.voice_position() != self.state.voice_position {
                    self.engine.set_voice_position(self.state.voice_position);
                }
                // Sync scale mode and interchange settings
                if self.engine.scale_mode() != self.scale_mode {
                    self.engine.set_scale_mode(self.scale_mode);
                }
                if self.engine.interchange_enabled() != self.interchange_enabled {
                    self.engine.set_interchange_enabled(self.interchange_enabled);
                }
                if self.engine.borrowing_range() != self.borrowing_range {
                    self.engine.set_borrowing_range(self.borrowing_range);
                }

                // Sync humanize config to wasm_humanizer each frame
                self.wasm_humanizer.update_config(self.humanize_config.clone());
                self.wasm_metronome.enabled = self.humanize_config.metronome_enabled;

                // Start beat clock on first running frame
                if !self.wasm_clock_started {
                    self.wasm_humanizer.clock_mut().start(now_ms);
                    self.wasm_clock_started = true;
                }

                // Tick the beat clock
                self.wasm_humanizer.tick(now_ms);

                // Check for metronome beat crossings
                if self.wasm_metronome.enabled {
                    if let Some(beat) = self.wasm_humanizer.clock().beat_crossed() {
                        let click_on = self.wasm_metronome.generate_click(beat);
                        let click_off = self.wasm_metronome.generate_click_off(beat);
                        let metro_port = self.humanize_config.metronome_output_port.unwrap_or(0);
                        let access = self.midi_access.borrow();
                        if let Some(ref access) = *access {
                            if let Some(id) = self.connected_output_ids.get(metro_port).or(self.connected_output_ids.first()) {
                                let _ = web::send_to_output(access, id, &click_on);
                                let _ = web::send_to_output(access, id, &click_off);
                            }
                        }
                    }
                }

                // Drain delayed notes from the delay queue
                let ready_notes = self.wasm_delay_queue.drain_ready(now_ms);
                let access = self.midi_access.borrow();
                if let Some(ref access) = *access {
                    for hn in ready_notes {
                        let port = hn.port;
                        if port < self.connected_output_ids.len() {
                            let midi_msg = if hn.is_note_off {
                                MidiMessage::NoteOff(hn.channel, hn.note, hn.velocity)
                            } else {
                                MidiMessage::NoteOn(hn.channel, hn.note, hn.velocity)
                            };
                            let mut buf = vec![0u8; midi_msg.bytes_size()];
                            if midi_msg.copy_to_slice(&mut buf).is_ok() {
                                let _ = web::send_to_output(access, &self.connected_output_ids[port], &buf);
                            }
                        }
                    }
                }
                drop(access);

                // Tick note generator and process events through harmony pipeline
                if self.generator_enabled {
                    let beat_pos = self.wasm_humanizer.clock().beat_position();
                    let bpm = self.humanize_config.bpm;
                    let gen_events = self.generator.tick(beat_pos, bpm);
                    for event in gen_events {
                        match event {
                            crate::generator::GeneratorEvent::NoteOn(note, vel) => {
                                let velocity = Velocity::try_from(vel).unwrap_or(Velocity::MAX);
                                self.handle_wasm_note_on(Channel::Ch1, note, velocity);
                            }
                            crate::generator::GeneratorEvent::NoteOff(note) => {
                                self.handle_wasm_note_off(Channel::Ch1, note, Velocity::try_from(0u8).unwrap());
                            }
                        }
                    }
                }

                // Process computer keyboard input (WASM)
                if self.keyboard_enabled {
                    let (presses, releases, focus_lost, keys_down) = ctx.input(|input| {
                        let mut kp = Vec::new();
                        let mut kr = Vec::new();
                        for event in &input.events {
                            match event {
                                egui::Event::Key { key, pressed: true, repeat: false, .. } => {
                                    if let Some(midi) = Self::key_to_midi_note(*key) {
                                        kp.push(midi);
                                    }
                                }
                                egui::Event::Key { key, pressed: false, .. } => {
                                    if let Some(midi) = Self::key_to_midi_note(*key) {
                                        kr.push(midi);
                                    }
                                }
                                _ => {}
                            }
                        }
                        // Collect currently held keys for reconciliation
                        let down: std::collections::HashSet<u8> = input.keys_down.iter()
                            .filter_map(|k| Self::key_to_midi_note(*k))
                            .collect();
                        let lost = !input.focused;
                        (kp, kr, lost, down)
                    });
                    for midi in presses {
                        if self.keyboard_held_keys.insert(midi) {
                            let note = Note::from_u8_lossy(midi);
                            let vel = Velocity::try_from(100u8).unwrap();
                            self.handle_wasm_note_on(Channel::Ch1, note, vel);
                        }
                    }
                    for midi in releases {
                        if self.keyboard_held_keys.remove(&midi) {
                            let note = Note::from_u8_lossy(midi);
                            let vel = Velocity::try_from(0u8).unwrap();
                            self.handle_wasm_note_off(Channel::Ch1, note, vel);
                        }
                    }
                    // Reconcile: release any keys we think are held but aren't actually down
                    // This catches missed keyup events (common in WASM/browser)
                    let stale: Vec<u8> = self.keyboard_held_keys.iter()
                        .filter(|midi| !keys_down.contains(midi))
                        .copied()
                        .collect();
                    for midi in stale {
                        self.keyboard_held_keys.remove(&midi);
                        let note = Note::from_u8_lossy(midi);
                        let vel = Velocity::try_from(0u8).unwrap();
                        self.handle_wasm_note_off(Channel::Ch1, note, vel);
                    }
                    // On focus loss, release everything
                    if focus_lost {
                        let all_held: Vec<u8> = self.keyboard_held_keys.drain().collect();
                        for midi in all_held {
                            let note = Note::from_u8_lossy(midi);
                            let vel = Velocity::try_from(0u8).unwrap();
                            self.handle_wasm_note_off(Channel::Ch1, note, vel);
                        }
                    }
                }

                // Process incoming MIDI messages
                let messages: Vec<Vec<u8>> = self.midi_queue.borrow_mut().drain(..).collect();
                for msg_bytes in messages {
                    self.process_wasm_midi(&msg_bytes);
                }
                ctx.request_repaint(); // continuous polling
            }
        }

        // Handle deferred start/stop from Play tab
        if self.pending_stop {
            self.pending_stop = false;
            self.stop();
        }
        if self.pending_start {
            self.pending_start = false;
            self.try_start(ctx);
        }

        // Sync humanize config, voice leading, and key/mode/octave to router shared state each frame
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.state.is_running {
                if let Some(ref router_state) = self.router_state {
                    if let Ok(mut state_lock) = router_state.lock() {
                        state_lock.humanize_config = self.humanize_config.clone();
                        state_lock.voice_leading_enabled = self.voice_leading_enabled;
                        state_lock.voice_leading_style = self.voice_leading_style;
                        state_lock.key = Some(self.state.key);
                        state_lock.mode = Some(self.state.mode);
                        state_lock.octave_mode = Some(self.state.octave_mode);
                        state_lock.voice_position = Some(self.state.voice_position);
                        state_lock.scale_mode = Some(self.scale_mode);
                        state_lock.interchange_enabled = Some(self.interchange_enabled);
                        state_lock.borrowing_range = Some(self.borrowing_range);
                        // Sync generator state
                        state_lock.generator_enabled = self.generator_enabled;
                        state_lock.generator_velocity = self.generator.velocity();
                        state_lock.generator_note_duration_beats = self.generator.note_duration_beats();
                        state_lock.keyboard_enabled = self.keyboard_enabled;
                    }
                }
            }

            // Process keyboard input for native (push events to router state)
            if self.keyboard_enabled {
                let (presses, releases, keys_down, focus_lost) = ctx.input(|input| {
                    let mut kp = Vec::new();
                    let mut kr = Vec::new();
                    for event in &input.events {
                        match event {
                            egui::Event::Key { key, pressed: true, repeat: false, .. } => {
                                if let Some(midi) = Self::key_to_midi_note(*key) {
                                    kp.push(midi);
                                }
                            }
                            egui::Event::Key { key, pressed: false, .. } => {
                                if let Some(midi) = Self::key_to_midi_note(*key) {
                                    kr.push(midi);
                                }
                            }
                            _ => {}
                        }
                    }
                    let down: std::collections::HashSet<u8> = input.keys_down.iter()
                        .filter_map(|k| Self::key_to_midi_note(*k))
                        .collect();
                    let lost = !input.focused;
                    (kp, kr, down, lost)
                });
                let mut kbd_events: Vec<(bool, wmidi::Note)> = Vec::new();
                for midi in presses {
                    if self.keyboard_held_keys.insert(midi) {
                        kbd_events.push((true, wmidi::Note::from_u8_lossy(midi)));
                    }
                }
                for midi in releases {
                    if self.keyboard_held_keys.remove(&midi) {
                        kbd_events.push((false, wmidi::Note::from_u8_lossy(midi)));
                    }
                }
                // Reconcile: release any keys we think are held but aren't actually down
                let stale: Vec<u8> = self.keyboard_held_keys.iter()
                    .filter(|midi| !keys_down.contains(midi))
                    .copied()
                    .collect();
                for midi in stale {
                    self.keyboard_held_keys.remove(&midi);
                    kbd_events.push((false, wmidi::Note::from_u8_lossy(midi)));
                }
                // On focus loss, release everything
                if focus_lost {
                    let all_held: Vec<u8> = self.keyboard_held_keys.drain().collect();
                    for midi in all_held {
                        kbd_events.push((false, wmidi::Note::from_u8_lossy(midi)));
                    }
                }
                if !kbd_events.is_empty() {
                    if let Some(ref router_state) = self.router_state {
                        if let Ok(mut state_lock) = router_state.lock() {
                            state_lock.keyboard_events.extend(kbd_events);
                        }
                    }
                }
            }
        }

        // Piano keyboard in BottomPanel (always visible)
        egui::TopBottomPanel::bottom("piano_panel").show(ctx, |ui| {
            let (input_notes, harmony_notes) = self.get_router_notes();

            // Chord detection
            let all_notes: HashSet<u8> = {
                let mut combined = input_notes.clone();
                combined.extend(&harmony_notes);
                combined
            };
            ui.vertical_centered(|ui| {
                let key_tonic = Some(self.state.key.semitones_from_c());
                ui.label(
                    egui::RichText::new(chord_display_with_analysis(&all_notes, key_tonic))
                        .size(24.0)
                        .strong()
                );
                // Borrowed-from indicator
                if self.interchange_enabled {
                    #[cfg(target_arch = "wasm32")]
                    let borrowed = self.engine.last_borrowed_from();
                    #[cfg(not(target_arch = "wasm32"))]
                    let borrowed: Option<ScaleMode> = None; // Router owns engine on native
                    if let Some(mode) = borrowed {
                        ui.label(
                            egui::RichText::new(format!("borrowed from {} {}", self.state.key, mode))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(204, 153, 0))
                        );
                    }
                }
            });

            ui.add_space(5.0);

            let gen_selected: HashSet<u8> = self.generator_selected_notes.iter().copied().collect();
            let gen_enabled = self.generator_enabled;
            // Compute in-scale pitch classes for piano tinting
            let in_scale_pcs: HashSet<u8> = {
                let tonic = self.state.key.semitones_from_c();
                let intervals = self.scale_mode.intervals();
                intervals.iter().map(|&offset| (tonic + offset) % 12).collect()
            };
            let borrowed_active = self.interchange_enabled && !harmony_notes.is_empty();
            let toggled = PianoKeyboard::new()
                .with_notes(input_notes, harmony_notes)
                .with_scale_tint(in_scale_pcs, borrowed_active)
                .with_selectable(gen_selected, gen_enabled)
                .show(ui);

            // Handle click-to-select toggles for generator
            for midi in toggled {
                if let Some(pos) = self.generator_selected_notes.iter().position(|&n| n == midi) {
                    self.generator_selected_notes.remove(pos);
                } else {
                    self.generator_selected_notes.push(midi);
                    self.generator_selected_notes.sort();
                }
                // Sync to generator engine
                let notes: Vec<wmidi::Note> = self.generator_selected_notes.iter()
                    .map(|&n| wmidi::Note::from_u8_lossy(n))
                    .collect();
                let _events = self.generator.set_selected_notes(notes);
            }

            ui.add_space(5.0);
        });

        // CentralPanel with single-screen layout
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_main_ui(ui, ctx);
        });

        // --- Render retro pixel overlay on foreground layer (non-interactive) ---
        {
            let screen = ctx.screen_rect();
            let fg_painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("ambient_overlay"),
            ));

            // Pixel-art decorative frame
            draw_ornate_frame(&fg_painter, screen.shrink(2.0));

            // CRT scanline overlay
            draw_scanlines(&fg_painter, screen);
        }
    }
}
