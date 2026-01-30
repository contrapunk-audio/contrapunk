//! GUI application for Contrapunk MIDI harmony generator.
//!
//! This module provides the eframe/egui-based GUI interface.

use std::collections::HashSet;
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
#[cfg(not(target_arch = "wasm32"))]
use crate::midi::ports::{list_input_ports, list_output_ports};
#[cfg(not(target_arch = "wasm32"))]
use crate::router::{spawn_gui_router, GUIRouterState};
#[cfg(target_arch = "wasm32")]
use crate::midi::web;
#[cfg(target_arch = "wasm32")]
use wmidi::{MidiMessage, Note, Channel, Velocity};

/// Converts a MIDI note number to a note name string (e.g., 60 -> "C4").
fn midi_to_name(midi: u8) -> String {
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
    state: AppState,
    /// Shared state for router communication
    #[cfg(not(target_arch = "wasm32"))]
    router_state: Option<Arc<Mutex<GUIRouterState>>>,
    /// Handle to the router thread
    #[cfg(not(target_arch = "wasm32"))]
    router_handle: Option<JoinHandle<Result<()>>>,
    /// Web MIDI access object (WASM only)
    #[cfg(target_arch = "wasm32")]
    midi_access: Rc<RefCell<Option<web_sys::MidiAccess>>>,
    /// Shared MIDI message queue — input callback pushes, update() drains (WASM only)
    #[cfg(target_arch = "wasm32")]
    midi_queue: Rc<RefCell<Vec<Vec<u8>>>>,
    /// Selected output device IDs for Web MIDI (WASM only)
    #[cfg(target_arch = "wasm32")]
    connected_output_ids: Vec<String>,
    /// Whether Web MIDI initialization has been attempted (WASM only)
    #[cfg(target_arch = "wasm32")]
    midi_initialized: bool,
    /// Harmony engine for WASM frame-based processing
    #[cfg(target_arch = "wasm32")]
    engine: HarmonyEngine,
    /// Active input notes for WASM display
    #[cfg(target_arch = "wasm32")]
    wasm_input_notes: HashSet<u8>,
    /// Active harmony notes for WASM display
    #[cfg(target_arch = "wasm32")]
    wasm_harmony_notes: HashSet<u8>,
    /// Whether voice leading is enabled
    voice_leading_enabled: bool,
    /// Current voice leading style
    voice_leading_style: VoiceLeadingStyle,
    /// Local copy of humanization config for GUI editing
    humanize_config: HumanizeConfig,
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
        };

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
    fn is_running(&self) -> bool {
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
    fn get_router_notes(&self) -> (HashSet<u8>, HashSet<u8>) {
        if let Some(ref router_state) = self.router_state {
            if let Ok(state) = router_state.lock() {
                return (state.input_notes.clone(), state.harmony_notes.clone());
            }
        }
        (HashSet::new(), HashSet::new())
    }

    /// Gets the current input/harmony notes (WASM).
    #[cfg(target_arch = "wasm32")]
    fn get_router_notes(&self) -> (HashSet<u8>, HashSet<u8>) {
        (self.wasm_input_notes.clone(), self.wasm_harmony_notes.clone())
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        // SidePanel with configuration controls (must be before CentralPanel)
        egui::SidePanel::left("config_panel")
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.heading("Contrapunk");
                ui.add_space(5.0);

                // Refresh Devices button
                if ui.button("Refresh Devices").clicked() {
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.midi_initialized = false;
                    }
                    self.state.refresh_devices();
                }

                // WASM MIDI status
                #[cfg(target_arch = "wasm32")]
                {
                    if self.midi_access.borrow().is_none() {
                        ui.colored_label(egui::Color32::YELLOW, "Requesting MIDI access...");
                    } else if self.state.available_inputs.is_empty() && self.state.available_outputs.is_empty() {
                        ui.colored_label(egui::Color32::YELLOW, "No MIDI devices found.");
                    }
                }

                ui.add_space(5.0);

                // Start/Stop button (moved to top for visibility)
                let start_clicked;
                let stop_clicked;
                if self.is_running() {
                    stop_clicked = ui.add_sized([180.0, 35.0], egui::Button::new("Stop")).clicked();
                    start_clicked = false;
                } else {
                    start_clicked = ui.add_sized([180.0, 35.0], egui::Button::new("Start")).clicked();
                    stop_clicked = false;
                }
                if stop_clicked {
                    self.stop();
                }
                if start_clicked {
                    self.try_start(ctx);
                }

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                // --- MIDI Configuration section ---
                egui::CollapsingHeader::new("MIDI Configuration")
                    .default_open(true)
                    .show(ui, |ui| {
                    // MIDI Input selection
                    ui.label("Input:");
                    let input_text = match self.state.input_port {
                        Some(idx) => self.state.available_inputs
                            .iter()
                            .find(|(i, _)| *i == idx)
                            .map(|(_, name)| name.clone())
                            .unwrap_or_else(|| format!("Port {}", idx)),
                        None => "Select input...".to_string(),
                    };
                    egui::ComboBox::from_id_salt("input_port")
                        .selected_text(&input_text)
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for (idx, name) in &self.state.available_inputs {
                                let is_selected = self.state.input_port == Some(*idx);
                                if ui.selectable_label(is_selected, name).clicked() {
                                    self.state.input_port = Some(*idx);
                                }
                            }
                        });
                    ui.add_space(5.0);

                    // MIDI Outputs selection
                    ui.label("Outputs:");
                    let num_slots = self.state.output_slots.len();
                    for slot_idx in 0..num_slots {
                        let slot_label = format!("Out {}", slot_idx + 1);
                        let output_text = match self.state.output_slots[slot_idx] {
                            Some(idx) => self.state.available_outputs
                                .iter()
                                .find(|(i, _)| *i == idx)
                                .map(|(_, name)| name.clone())
                                .unwrap_or_else(|| format!("Port {}", idx)),
                            None => "None".to_string(),
                        };

                        ui.horizontal(|ui| {
                            ui.label(&slot_label);
                        });
                        egui::ComboBox::from_id_salt(format!("output_slot_{}", slot_idx))
                            .selected_text(&output_text)
                            .width(160.0)
                            .show_ui(ui, |ui| {
                                let is_none = self.state.output_slots[slot_idx].is_none();
                                if ui.selectable_label(is_none, "None").clicked() {
                                    self.state.output_slots[slot_idx] = None;
                                }
                                for (idx, name) in &self.state.available_outputs {
                                    let is_selected = self.state.output_slots[slot_idx] == Some(*idx);
                                    if ui.selectable_label(is_selected, name).clicked() {
                                        self.state.output_slots[slot_idx] = Some(*idx);
                                    }
                                }
                            });
                        ui.add_space(2.0);
                    }
                });

                // --- Harmony section ---
                egui::CollapsingHeader::new("Harmony")
                    .default_open(true)
                    .show(ui, |ui| {
                    ui.label("Key:");
                    egui::ComboBox::from_id_salt("key_select")
                        .selected_text(format!("{}", self.state.key))
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for key in Key::all() {
                                ui.selectable_value(&mut self.state.key, *key, format!("{}", key));
                            }
                        });
                    ui.add_space(5.0);

                    ui.label("Mode:");
                    egui::ComboBox::from_id_salt("mode_select")
                        .selected_text(format!("{}: {}", self.state.mode.number(), self.state.mode.description()))
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for mode in HarmonyMode::all() {
                                let text = format!("{}: {}", mode.number(), mode.description());
                                ui.selectable_value(&mut self.state.mode, *mode, text);
                            }
                        });
                    ui.add_space(5.0);

                    ui.label("Octave:");
                    egui::ComboBox::from_id_salt("octave_mode_select")
                        .selected_text(self.state.octave_mode.description())
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for octave_mode in OctaveMode::all() {
                                ui.selectable_value(&mut self.state.octave_mode, *octave_mode, octave_mode.description());
                            }
                        });
                });

                // --- Voice Leading section ---
                egui::CollapsingHeader::new("Voice Leading")
                    .default_open(true)
                    .show(ui, |ui| {
                    ui.checkbox(&mut self.voice_leading_enabled, "Enable Voice Leading");
                    if self.voice_leading_enabled {
                        ui.add_space(5.0);
                        ui.label("Style:");
                        egui::ComboBox::from_id_salt("voice_leading_style")
                            .selected_text(self.voice_leading_style.description())
                            .width(160.0)
                            .show_ui(ui, |ui| {
                                for style in VoiceLeadingStyle::all() {
                                    ui.selectable_value(&mut self.voice_leading_style, *style, style.description());
                                }
                            });
                    }
                });

                // --- Metronome section ---
                egui::CollapsingHeader::new("Metronome")
                    .default_open(true)
                    .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut self.humanize_config.bpm, 40.0..=240.0).text("BPM"));
                    ui.label(format!("Time Sig: {}/{}", self.humanize_config.beats_per_bar, self.humanize_config.beat_unit));
                    ui.checkbox(&mut self.humanize_config.metronome_enabled, "Metronome Click");
                    if self.humanize_config.metronome_enabled {
                        let metro_port = self.humanize_config.metronome_output_port.unwrap_or(0);
                        let port_label = self.state.output_slots.get(metro_port)
                            .and_then(|s| *s)
                            .and_then(|idx| self.state.available_outputs.iter().find(|(i, _)| *i == idx))
                            .map(|(_, name)| name.clone())
                            .unwrap_or_else(|| format!("Out {}", metro_port + 1));
                        ui.label("Output:");
                        egui::ComboBox::from_id_salt("metronome_output")
                            .selected_text(&port_label)
                            .width(160.0)
                            .show_ui(ui, |ui| {
                                for (slot_idx, slot) in self.state.output_slots.iter().enumerate() {
                                    if slot.is_some() {
                                        let name = slot
                                            .and_then(|idx| self.state.available_outputs.iter().find(|(i, _)| *i == idx))
                                            .map(|(_, name)| name.clone())
                                            .unwrap_or_else(|| format!("Out {}", slot_idx + 1));
                                        let is_selected = self.humanize_config.metronome_output_port == Some(slot_idx)
                                            || (self.humanize_config.metronome_output_port.is_none() && slot_idx == 0);
                                        if ui.selectable_label(is_selected, &name).clicked() {
                                            self.humanize_config.metronome_output_port = Some(slot_idx);
                                        }
                                    }
                                }
                            });
                    }
                });

                // --- Humanization section ---
                egui::CollapsingHeader::new("Humanization")
                    .default_open(true)
                    .show(ui, |ui| {
                    ui.checkbox(&mut self.humanize_config.enabled, "Enable Humanization");

                    if self.humanize_config.enabled {
                        ui.add_space(5.0);

                        // Timing Jitter
                        egui::CollapsingHeader::new("Timing Jitter")
                            .default_open(false)
                            .show(ui, |ui| {
                            ui.checkbox(&mut self.humanize_config.jitter_enabled, "Enable Jitter");
                            if self.humanize_config.jitter_enabled {
                                ui.add(egui::Slider::new(&mut self.humanize_config.jitter_min_ms, 0..=50).text("Min ms"));
                                ui.add(egui::Slider::new(&mut self.humanize_config.jitter_max_ms, 0..=50).text("Max ms"));
                                if self.humanize_config.jitter_min_ms > self.humanize_config.jitter_max_ms {
                                    self.humanize_config.jitter_min_ms = self.humanize_config.jitter_max_ms;
                                }
                            }
                        });

                        // Velocity
                        egui::CollapsingHeader::new("Velocity")
                            .default_open(false)
                            .show(ui, |ui| {
                            ui.checkbox(&mut self.humanize_config.velocity_enabled, "Velocity Variation");
                            if self.humanize_config.velocity_enabled {
                                ui.add(egui::Slider::new(&mut self.humanize_config.velocity_variation, 0..=30).text("\u{00b1}"));
                            }
                        });

                        // Duration
                        egui::CollapsingHeader::new("Duration")
                            .default_open(false)
                            .show(ui, |ui| {
                            ui.checkbox(&mut self.humanize_config.duration_enabled, "Duration Variation");
                            if self.humanize_config.duration_enabled {
                                ui.add(egui::Slider::new(&mut self.humanize_config.duration_variation_ms, 0..=100).text("ms"));
                            }
                        });

                        // Swing/Groove
                        egui::CollapsingHeader::new("Swing/Groove")
                            .default_open(false)
                            .show(ui, |ui| {
                            ui.checkbox(&mut self.humanize_config.swing_enabled, "Swing");
                            if self.humanize_config.swing_enabled {
                                ui.add(egui::Slider::new(&mut self.humanize_config.swing_amount, 0.0..=1.0).text("Amount"));
                            }
                        });
                    }
                });

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
                }); // ScrollArea
            });

        // CentralPanel with status and visualization (after SidePanel)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Contrapunk");
                ui.add_space(5.0);
                ui.label("MIDI Harmony Generator");
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Status row
            ui.horizontal(|ui| {
                if self.is_running() {
                    ui.label(egui::RichText::new("ACTIVE").color(egui::Color32::GREEN).strong());
                } else {
                    ui.label(egui::RichText::new("STOPPED").color(egui::Color32::GRAY));
                }
                ui.separator();
                ui.label(format!("Key: {}", self.state.key));
                ui.separator();
                ui.label(format!("Mode: {}", self.state.mode.description()));
                if self.state.octave_mode != OctaveMode::None {
                    ui.separator();
                    ui.label(format!("Octave: {}", self.state.octave_mode.description()));
                }
                if self.voice_leading_enabled {
                    ui.separator();
                    ui.label(format!("Voice: {}", self.voice_leading_style.description()));
                }
            });

            ui.add_space(10.0);

            // Error display
            if let Some(ref error) = self.state.last_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                ui.add_space(10.0);
            }

            ui.separator();
            ui.add_space(10.0);

            // Connected devices info (when running)
            if self.is_running() {
                ui.label(egui::RichText::new("Connected Devices:").strong());
                ui.add_space(5.0);

                // Input device
                if let Some(input_idx) = self.state.input_port {
                    let input_name = self.state.available_inputs
                        .iter()
                        .find(|(i, _)| *i == input_idx)
                        .map(|(_, name)| name.as_str())
                        .unwrap_or("Unknown");
                    ui.label(format!("  Input: {}", input_name));
                }

                // Output devices
                let outputs = self.state.selected_output_ports();
                for (i, idx) in outputs.iter().enumerate() {
                    let output_name = self.state.available_outputs
                        .iter()
                        .find(|(port_idx, _)| port_idx == idx)
                        .map(|(_, name)| name.as_str())
                        .unwrap_or("Unknown");
                    let role = if i == 0 { "melody" } else { "harmony" };
                    ui.label(format!("  Output {}: {} [{}]", i + 1, output_name, role));
                }

            }

            // Active notes display (always shown, updates in real-time)
            ui.add_space(15.0);
            let (input_notes, harmony_notes) = self.get_router_notes();

            ui.group(|ui| {
                ui.label(egui::RichText::new("Active Notes").strong());

                ui.horizontal_wrapped(|ui| {
                    ui.label("Input: ");
                    if input_notes.is_empty() {
                        ui.label("(none)");
                    } else {
                        let mut sorted: Vec<_> = input_notes.iter().copied().collect();
                        sorted.sort();
                        for midi in sorted {
                            ui.label(
                                egui::RichText::new(midi_to_name(midi))
                                    .color(egui::Color32::LIGHT_BLUE)
                            );
                        }
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label("Harmony: ");
                    if harmony_notes.is_empty() {
                        ui.label("(none)");
                    } else {
                        let mut sorted: Vec<_> = harmony_notes.iter().copied().collect();
                        sorted.sort();
                        for midi in sorted {
                            ui.label(
                                egui::RichText::new(midi_to_name(midi))
                                    .color(egui::Color32::LIGHT_GREEN)
                            );
                        }
                    }
                });
            });

            // Chord detection display
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.label("Detected Chord");
                let all_notes: HashSet<u8> = {
                    let mut combined = input_notes.clone();
                    combined.extend(&harmony_notes);
                    combined
                };
                ui.label(
                    egui::RichText::new(chord_display(&all_notes))
                        .size(32.0)
                        .strong()
                );
            });

            ui.add_space(20.0);

            // Piano keyboard visualization
            ui.group(|ui| {
                ui.label(egui::RichText::new("Piano Keyboard").strong());
                ui.add_space(5.0);

                PianoKeyboard::new()
                    .with_notes(input_notes, harmony_notes)
                    .show(ui);
            });
        });
    }
}
