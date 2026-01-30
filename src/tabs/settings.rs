//! Settings tab: app configuration, MIDI device management.

use eframe::egui;
use crate::app::ContrapunkApp;

impl ContrapunkApp {
    /// Draws the Settings tab content: MIDI devices and app config.
    pub fn draw_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.add_space(8.0);

        // --- MIDI Devices ---
        egui::CollapsingHeader::new(egui::RichText::new("MIDI Devices").strong())
            .default_open(true)
            .show(ui, |ui| {
                if ui.button("Refresh Devices").clicked() {
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.midi_initialized = false;
                    }
                    self.state.refresh_devices();
                }
                ui.add_space(5.0);

                // Input
                ui.label("Input:");
                let input_text = match self.state.input_port {
                    Some(idx) => self.state.available_inputs
                        .iter()
                        .find(|(i, _)| *i == idx)
                        .map(|(_, name)| name.clone())
                        .unwrap_or_else(|| format!("Port {}", idx)),
                    None => "Select input...".to_string(),
                };
                egui::ComboBox::from_id_salt("settings_input_port")
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
                ui.add_space(5.0);

                // Outputs
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
                    egui::ComboBox::from_id_salt(format!("settings_output_slot_{}", slot_idx))
                        .selected_text(&output_text)
                        .width(250.0)
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

        ui.add_space(10.0);

        // --- App Info ---
        egui::CollapsingHeader::new(egui::RichText::new("About").strong())
            .default_open(false)
            .show(ui, |ui| {
                ui.label("Contrapunk - MIDI Harmony Generator");
                #[cfg(target_arch = "wasm32")]
                ui.label("Build: WASM (browser)");
                #[cfg(not(target_arch = "wasm32"))]
                ui.label("Build: Native");
                ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            });
    }
}
