//! Play tab: performance-focused controls.
//!
//! Shows key, mode, MIDI device selection, start/stop, active preset quick-switch,
//! and active notes display.

use eframe::egui;
use crate::app::{ContrapunkApp, midi_to_name};
use crate::harmony::{Key, HarmonyMode};

impl ContrapunkApp {
    /// Draws the Play tab content: simplified performance controls.
    pub fn draw_play_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Play");
        ui.add_space(8.0);

        // --- Start/Stop ---
        ui.horizontal(|ui| {
            if self.is_running() {
                if ui.add_sized([120.0, 35.0], egui::Button::new("Stop")).clicked() {
                    self.pending_stop = true;
                }
            } else {
                if ui.add_sized([120.0, 35.0], egui::Button::new("Start")).clicked() {
                    self.pending_start = true;
                }
            }

            // Status indicator
            ui.add_space(10.0);
            if self.is_running() {
                ui.label(egui::RichText::new("ACTIVE").color(egui::Color32::GREEN).strong());
            } else {
                ui.label(egui::RichText::new("STOPPED").color(egui::Color32::GRAY));
            }
        });
        ui.add_space(8.0);

        // --- Preset Quick-Switch ---
        ui.separator();
        ui.add_space(5.0);
        ui.label(egui::RichText::new("Preset").strong());
        // Show active preset persona prominently
        if let Some(preset) = self.preset_manager.active() {
            ui.label(
                egui::RichText::new(&preset.persona)
                    .size(18.0)
                    .strong()
                    .color(egui::Color32::from_rgb(218, 165, 32)) // gold
            );
            ui.label(
                egui::RichText::new(&preset.genre)
                    .size(12.0)
                    .italics()
                    .color(egui::Color32::GRAY)
            );
        }
        let active_name = self.preset_manager.active()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "(none)".to_string());
        // Collect preset info to avoid borrow conflicts
        let preset_info: Vec<(usize, String, bool)> = self.preset_manager.all_presets()
            .iter().enumerate()
            .map(|(i, p)| (i, format!("{} - {}", p.name, p.genre), p.name == active_name))
            .collect();
        let mut selected_preset: Option<usize> = None;
        egui::ComboBox::from_id_salt("play_preset_select")
            .selected_text(&active_name)
            .width(250.0)
            .show_ui(ui, |ui| {
                for (i, label, is_active) in &preset_info {
                    if ui.selectable_label(*is_active, label).clicked() {
                        selected_preset = Some(*i);
                    }
                }
            });
        if let Some(idx) = selected_preset {
            self.preset_manager.set_active(idx);
            self.apply_active_preset();
        }

        ui.add_space(10.0);

        // --- Key & Mode (quick access) ---
        ui.separator();
        ui.add_space(5.0);
        ui.label(egui::RichText::new("Harmony").strong());

        ui.horizontal(|ui| {
            ui.label("Key:");
            egui::ComboBox::from_id_salt("play_key")
                .selected_text(format!("{}", self.state.key))
                .width(80.0)
                .show_ui(ui, |ui| {
                    for key in Key::all() {
                        ui.selectable_value(&mut self.state.key, *key, format!("{}", key));
                    }
                });

            ui.add_space(10.0);
            ui.label("Mode:");
            egui::ComboBox::from_id_salt("play_mode")
                .selected_text(format!("{}", self.state.mode.description()))
                .width(150.0)
                .show_ui(ui, |ui| {
                    for mode in HarmonyMode::all() {
                        let text = format!("{}: {}", mode.number(), mode.description());
                        ui.selectable_value(&mut self.state.mode, *mode, text);
                    }
                });
        });

        ui.add_space(10.0);

        // --- MIDI Input Device ---
        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.separator();
            ui.add_space(5.0);
            ui.label(egui::RichText::new("MIDI Input").strong());
            let input_text = match self.state.input_port {
                Some(idx) => self.state.available_inputs
                    .iter()
                    .find(|(i, _)| *i == idx)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| format!("Port {}", idx)),
                None => "Select input...".to_string(),
            };
            egui::ComboBox::from_id_salt("play_input_port")
                .selected_text(&input_text)
                .width(250.0)
                .show_ui(ui, |ui| {
                    for (idx, name) in &self.state.available_inputs {
                        let is_selected = self.state.input_port == Some(*idx);
                        if ui.selectable_label(is_selected, name).clicked() {
                            self.state.input_port = Some(*idx);
                        }
                    }
                });
        }

        #[cfg(target_arch = "wasm32")]
        {
            ui.separator();
            ui.add_space(5.0);
            ui.label(egui::RichText::new("MIDI Input").strong());
            if self.midi_access.borrow().is_none() {
                ui.colored_label(egui::Color32::YELLOW, "Requesting MIDI access...");
            } else if self.state.available_inputs.is_empty() {
                ui.colored_label(egui::Color32::YELLOW, "No MIDI devices found.");
            } else {
                let input_text = match self.state.input_port {
                    Some(idx) => self.state.available_inputs
                        .iter()
                        .find(|(i, _)| *i == idx)
                        .map(|(_, name)| name.clone())
                        .unwrap_or_else(|| format!("Port {}", idx)),
                    None => "Select input...".to_string(),
                };
                egui::ComboBox::from_id_salt("play_input_port")
                    .selected_text(&input_text)
                    .width(250.0)
                    .show_ui(ui, |ui| {
                        for (idx, name) in &self.state.available_inputs {
                            let is_selected = self.state.input_port == Some(*idx);
                            if ui.selectable_label(is_selected, name).clicked() {
                                self.state.input_port = Some(*idx);
                            }
                        }
                    });
            }
        }

        ui.add_space(10.0);

        // --- Active Notes ---
        ui.separator();
        ui.add_space(5.0);
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

        // Error display
        if let Some(ref error) = self.state.last_error {
            ui.add_space(5.0);
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }
    }
}
