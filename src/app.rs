//! GUI application for Contrapunk MIDI harmony generator.
//!
//! This module provides the eframe/egui-based GUI interface.

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use anyhow::Result;
use eframe::egui;

use crate::harmony::{Key, HarmonyMode};
use crate::midi::ports::{list_input_ports, list_output_ports};
use crate::router::{spawn_gui_router, GUIRouterState};

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
    router_state: Option<Arc<Mutex<GUIRouterState>>>,
    /// Handle to the router thread
    router_handle: Option<JoinHandle<Result<()>>>,
}

impl ContrapunkApp {
    /// Create a new ContrapunkApp instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            state: AppState::default(),
            router_state: None,
            router_handle: None,
        };
        // Auto-refresh devices on startup
        app.state.refresh_devices();
        app
    }

    /// Returns whether routing is currently active.
    fn is_running(&self) -> bool {
        self.state.is_running
    }

    /// Validates configuration and attempts to start routing.
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

    /// Stops routing.
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

    /// Gets the current input/harmony notes from the router state.
    fn get_router_notes(&self) -> (usize, usize) {
        if let Some(ref router_state) = self.router_state {
            if let Ok(state) = router_state.lock() {
                return (state.input_notes.len(), state.harmony_notes.len());
            }
        }
        (0, 0)
    }
}

impl eframe::App for ContrapunkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // SidePanel with configuration controls (must be before CentralPanel)
        egui::SidePanel::left("config_panel")
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Configuration");
                ui.add_space(10.0);

                // Refresh Devices button
                if ui.button("Refresh Devices").clicked() {
                    self.state.refresh_devices();
                }
                ui.add_space(15.0);

                // MIDI Input selection
                ui.label("MIDI Input:");
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
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for (idx, name) in &self.state.available_inputs {
                            let is_selected = self.state.input_port == Some(*idx);
                            if ui.selectable_label(is_selected, name).clicked() {
                                self.state.input_port = Some(*idx);
                            }
                        }
                    });
                ui.add_space(15.0);

                // MIDI Outputs selection (multiple slots)
                ui.label("MIDI Outputs:");
                ui.add_space(5.0);

                let num_slots = self.state.output_slots.len();
                for slot_idx in 0..num_slots {
                    let slot_label = format!("Output {}", slot_idx + 1);
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
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            // "None" option to clear slot
                            let is_none = self.state.output_slots[slot_idx].is_none();
                            if ui.selectable_label(is_none, "None").clicked() {
                                self.state.output_slots[slot_idx] = None;
                            }
                            // Available outputs
                            for (idx, name) in &self.state.available_outputs {
                                let is_selected = self.state.output_slots[slot_idx] == Some(*idx);
                                if ui.selectable_label(is_selected, name).clicked() {
                                    self.state.output_slots[slot_idx] = Some(*idx);
                                }
                            }
                        });
                    ui.add_space(3.0);
                }
                ui.add_space(15.0);

                ui.separator();
                ui.add_space(10.0);

                // Key selection
                ui.label("Musical Key:");
                egui::ComboBox::from_id_salt("key_select")
                    .selected_text(format!("{}", self.state.key))
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for key in Key::all() {
                            ui.selectable_value(&mut self.state.key, *key, format!("{}", key));
                        }
                    });
                ui.add_space(10.0);

                // Mode selection
                ui.label("Harmony Mode:");
                egui::ComboBox::from_id_salt("mode_select")
                    .selected_text(format!("{}: {}", self.state.mode.number(), self.state.mode.description()))
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for mode in HarmonyMode::all() {
                            let text = format!("{}: {}", mode.number(), mode.description());
                            ui.selectable_value(&mut self.state.mode, *mode, text);
                        }
                    });
                ui.add_space(20.0);

                ui.separator();
                ui.add_space(10.0);

                // Start/Stop button
                let start_clicked;
                let stop_clicked;
                if self.is_running() {
                    stop_clicked = ui.add_sized([180.0, 40.0], egui::Button::new("Stop")).clicked();
                    start_clicked = false;
                } else {
                    start_clicked = ui.add_sized([180.0, 40.0], egui::Button::new("Start")).clicked();
                    stop_clicked = false;
                }

                // Handle button clicks after UI borrowing ends
                if stop_clicked {
                    self.stop();
                }
                if start_clicked {
                    self.try_start(ctx);
                }
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

                // Note activity indicator
                ui.add_space(15.0);
                let (input_count, harmony_count) = self.get_router_notes();
                if input_count > 0 || harmony_count > 0 {
                    ui.label(egui::RichText::new("Active Notes:").strong());
                    ui.label(format!("  Input: {} notes", input_count));
                    ui.label(format!("  Harmony: {} notes", harmony_count));
                }
            }

            ui.add_space(20.0);

            // Placeholder for future visualizations
            ui.label("Note visualizations will be added in subsequent plans.");
        });
    }
}
