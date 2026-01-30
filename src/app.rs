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
use crate::chord::chord_display;
use crate::harmony::{Key, HarmonyMode, OctaveMode, HarmonyEngine, VoiceLeadingStyle};
use crate::humanize::HumanizeConfig;
#[cfg(target_arch = "wasm32")]
use crate::humanize::{Humanizer, DelayQueue, Metronome};
use crate::piano::PianoKeyboard;
use crate::preset::{PresetManager, storage as preset_storage};
use crate::theme::ContrapunkTheme;
use crate::theme::widgets::{draw_gear, draw_ornate_frame};
use crate::theme::colors::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::midi::ports::{list_input_ports, list_output_ports};
#[cfg(not(target_arch = "wasm32"))]
use crate::router::{spawn_gui_router, GUIRouterState};
#[cfg(target_arch = "wasm32")]
use crate::midi::web;
#[cfg(target_arch = "wasm32")]
use wmidi::{MidiMessage, Note, Channel, Velocity};

/// Tab navigation for the main UI.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tab {
    Play,
    Craft,
    Settings,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Play => "Play",
            Tab::Craft => "Craft",
            Tab::Settings => "Settings",
        }
    }
}

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
    /// Currently active tab
    pub(crate) active_tab: Tab,
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
    /// Whether voice leading is enabled
    pub(crate) voice_leading_enabled: bool,
    /// Current voice leading style
    pub(crate) voice_leading_style: VoiceLeadingStyle,
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
}

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
            active_tab: Tab::Play,
            preset_manager: PresetManager::new(),
            pending_start: false,
            pending_stop: false,
            theme_applied: false,
            note_activity: 0.0,
            prev_note_count: 0,
            voice_leading_enabled: false,
            voice_leading_style: VoiceLeadingStyle::default(),
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

        let access = self.midi_access.borrow();
        let access = match access.as_ref() {
            Some(a) => a,
            None => {
                self.state.last_error = Some("Web MIDI not available. Use Chrome/Edge with MIDI device.".to_string());
                return;
            }
        };

        // Validate input port is selected
        let input_idx = match self.state.input_port {
            Some(idx) => idx,
            None => {
                self.state.last_error = Some("Please select an input device".to_string());
                return;
            }
        };

        // Get input device ID
        let inputs = web::list_midi_inputs(access);
        let input_id = match inputs.get(input_idx) {
            Some((id, _)) => id.clone(),
            None => {
                self.state.last_error = Some("Selected input device not found".to_string());
                return;
            }
        };

        // Validate at least one output is selected
        let output_ports = self.state.selected_output_ports();
        if output_ports.is_empty() {
            self.state.last_error = Some("Please select at least one output device".to_string());
            return;
        }

        // Collect output device IDs
        let outputs = web::list_midi_outputs(access);
        self.connected_output_ids.clear();
        for idx in &output_ports {
            if let Some((id, _)) = outputs.get(*idx) {
                self.connected_output_ids.push(id.clone());
            }
        }

        // Connect input callback
        if let Err(_e) = web::connect_input(access, &input_id, self.midi_queue.clone()) {
            self.state.last_error = Some("Failed to connect to MIDI input".to_string());
            return;
        }

        // Configure harmony engine
        self.engine = HarmonyEngine::new(self.state.key, self.state.mode);
        let num_outputs = self.connected_output_ids.len();
        self.engine.set_voice_count(num_outputs);
        self.engine.set_octave_mode(self.state.octave_mode);

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

    /// Applies the currently active preset's settings to app state.
    pub(crate) fn apply_active_preset(&mut self) {
        if let Some(preset) = self.preset_manager.active().cloned() {
            self.state.key = preset.key;
            self.state.mode = preset.harmony_mode;
            self.state.octave_mode = preset.octave_mode;
            self.voice_leading_enabled = preset.voice_leading_enabled;
            self.voice_leading_style = preset.voice_leading_style;
            self.humanize_config = preset.humanize_config;
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

        // --- Ambient background animations ---
        {
            let time = ctx.input(|i| i.time) as f32;
            let screen = ctx.screen_rect();
            let bg_painter = ctx.layer_painter(egui::LayerId::background());

            // Update note activity from current notes
            let (input_n, harmony_n) = self.get_router_notes();
            let current_count = input_n.len() + harmony_n.len();
            if current_count > 0 {
                self.note_activity = 1.0;
            } else {
                self.note_activity *= 0.95;
            }
            let activity = self.note_activity;
            self.prev_note_count = current_count;

            // Gear positions (corners and edges)
            let gear_specs: [(egui::Pos2, f32, f32); 5] = [
                (egui::pos2(screen.min.x + 30.0, screen.min.y + 30.0), 18.0, 0.3),
                (egui::pos2(screen.max.x - 30.0, screen.min.y + 30.0), 14.0, -0.2),
                (egui::pos2(screen.min.x + 25.0, screen.max.y - 25.0), 16.0, 0.25),
                (egui::pos2(screen.max.x - 25.0, screen.max.y - 25.0), 15.0, -0.3),
                (egui::pos2(screen.center().x, screen.min.y + 20.0), 12.0, 0.15),
            ];

            for (pos, radius, base_speed) in &gear_specs {
                let speed = base_speed + activity * 2.0 * base_speed.signum();
                let angle = time * speed;
                let a = activity.min(1.0);
                let gear_color = egui::Color32::from_rgba_premultiplied(
                    (WIDGET_INACTIVE.r() as f32 + (COPPER.r() as f32 - WIDGET_INACTIVE.r() as f32) * a) as u8,
                    (WIDGET_INACTIVE.g() as f32 + (COPPER.g() as f32 - WIDGET_INACTIVE.g() as f32) * a) as u8,
                    (WIDGET_INACTIVE.b() as f32 + (COPPER.b() as f32 - WIDGET_INACTIVE.b() as f32) * a) as u8,
                    70,
                );
                if activity > 0.3 {
                    let glow_alpha = ((activity - 0.3) * 60.0).min(40.0) as u8;
                    bg_painter.circle_filled(
                        *pos,
                        radius * 1.8,
                        egui::Color32::from_rgba_premultiplied(
                            COPPER.r(), COPPER.g(), COPPER.b(), glow_alpha,
                        ),
                    );
                }
                draw_gear(&bg_painter, *pos, *radius, 10, angle, gear_color);
            }

            // Floating particles
            for i in 0..8 {
                let seed = i as f32 * 1.7;
                let x_base = screen.min.x + (screen.width() * (0.1 + 0.1 * i as f32));
                let y_period = screen.height();
                let y = screen.max.y - ((time * 0.5 + seed * 3.0) % y_period);
                let x = x_base + (time * 0.3 + seed).sin() * 20.0;
                let alpha = 30 + (activity * 20.0) as u8;
                bg_painter.circle_filled(
                    egui::pos2(x, y),
                    1.5,
                    egui::Color32::from_rgba_premultiplied(GOLD.r(), GOLD.g(), GOLD.b(), alpha),
                );
            }

            // Decorative frame
            draw_ornate_frame(&bg_painter, screen.shrink(2.0));

            // Request repaint at 30fps for ambient animations
            ctx.request_repaint_after(Duration::from_millis(33));
        }

        // Tab bar at the top
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for tab in &[Tab::Play, Tab::Craft, Tab::Settings] {
                    ui.selectable_value(&mut self.active_tab, *tab, tab.label());
                }
            });
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

        // Sync humanize config and voice leading to router shared state each frame
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.state.is_running {
                if let Some(ref router_state) = self.router_state {
                    if let Ok(mut state_lock) = router_state.lock() {
                        state_lock.humanize_config = self.humanize_config.clone();
                        state_lock.voice_leading_enabled = self.voice_leading_enabled;
                        state_lock.voice_leading_style = self.voice_leading_style;
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
                ui.label(
                    egui::RichText::new(chord_display(&all_notes))
                        .size(24.0)
                        .strong()
                );
            });

            ui.add_space(5.0);

            PianoKeyboard::new()
                .with_notes(input_notes, harmony_notes)
                .show(ui);

            ui.add_space(5.0);
        });

        // CentralPanel with tab content routing
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Play => self.draw_play_tab(ui, ctx),
                Tab::Craft => self.draw_craft_tab(ui),
                Tab::Settings => self.draw_settings_tab(ui),
            }
        });
    }
}
