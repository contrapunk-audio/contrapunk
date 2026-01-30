//! Craft tab: full parameter editor for harmony, humanization, and voice leading.
//!
//! Combines all parameter controls in one deep-editing workspace with ornate
//! steampunk-themed widgets.

use eframe::egui;
use crate::app::ContrapunkApp;
use crate::harmony::{Key, HarmonyMode, OctaveMode, VoiceLeadingStyle};
use crate::preset::storage::{export_preset_json, import_preset_json};
use crate::theme::widgets::{ornate_slider, ornate_toggle, section_header};

impl ContrapunkApp {
    /// Draws the Craft tab content: full parameter editor.
    pub fn draw_craft_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Craft");
        ui.add_space(8.0);

        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {

        // --- Preset Management ---
        section_header(ui, "Preset");
        {
            let active_display = self.preset_manager.active()
                .map(|p| format!("{} -- {} ({})", p.name, p.persona, p.genre))
                .unwrap_or_else(|| "(none)".to_string());
            let active_short = self.preset_manager.active()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Select...".to_string());
            ui.label(format!("Active: {}", active_display));
            ui.add_space(3.0);

            // Collect preset info to avoid borrow conflicts
            let preset_info: Vec<(usize, String, bool)> = self.preset_manager.all_presets()
                .iter().enumerate()
                .map(|(i, p)| (i, format!("{} - {}", p.name, p.genre), p.name == active_short))
                .collect();
            let mut selected_preset: Option<usize> = None;
            egui::ComboBox::from_id_salt("craft_preset_select")
                .selected_text(&active_short)
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

            ui.add_space(5.0);

            // Save As
            if ui.button("Save As...").clicked() {
                self.craft_save_as_open = !self.craft_save_as_open;
            }
            if self.craft_save_as_open {
                ui.group(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.craft_save_as_name);
                    ui.label("Persona:");
                    ui.text_edit_singleline(&mut self.craft_save_as_persona);
                    ui.label("Genre:");
                    ui.text_edit_singleline(&mut self.craft_save_as_genre);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() && !self.craft_save_as_name.is_empty() {
                            let preset = self.preset_from_current(
                                &self.craft_save_as_name.clone(),
                                &self.craft_save_as_persona.clone(),
                                &self.craft_save_as_genre.clone(),
                            );
                            self.preset_manager.add_custom(preset);
                            self.craft_save_as_open = false;
                            self.craft_save_as_name.clear();
                            self.craft_save_as_persona.clear();
                            self.craft_save_as_genre.clear();
                        }
                        if ui.button("Cancel").clicked() {
                            self.craft_save_as_open = false;
                        }
                    });
                });
            }

            // Create New (quick save with default name)
            if ui.button("Create New").clicked() {
                let count = self.preset_manager.custom_presets().len() + 1;
                let name = format!("Custom {}", count);
                let preset = self.preset_from_current(&name, "Custom", "User");
                self.preset_manager.add_custom(preset);
            }

            // Delete custom preset (only for non-builtin)
            let active_is_custom = self.preset_manager.active()
                .map(|p| !p.is_builtin)
                .unwrap_or(false);
            if active_is_custom {
                if ui.button("Delete Active Preset").clicked() {
                    // Find custom index from active index
                    if let Some(active) = self.preset_manager.active().cloned() {
                        let custom_idx = self.preset_manager.custom_presets()
                            .iter()
                            .position(|p| p.name == active.name);
                        if let Some(idx) = custom_idx {
                            self.preset_manager.remove_custom(idx);
                        }
                    }
                }
            }

            ui.add_space(3.0);

            // Export JSON
            if ui.button("Export JSON").clicked() {
                if let Some(preset) = self.preset_manager.active() {
                    self.craft_export_json = export_preset_json(preset);
                }
            }
            if !self.craft_export_json.is_empty() {
                ui.group(|ui| {
                    ui.label("Exported JSON (copy):");
                    egui::ScrollArea::vertical()
                        .id_salt("export_json_scroll")
                        .max_height(120.0)
                        .show(ui, |ui| {
                            ui.add(egui::TextEdit::multiline(&mut self.craft_export_json)
                                .desired_width(f32::INFINITY)
                                .desired_rows(4));
                        });
                    if ui.button("Close").clicked() {
                        self.craft_export_json.clear();
                    }
                });
            }

            // Import JSON
            if ui.button("Import JSON").clicked() {
                self.craft_import_open = !self.craft_import_open;
            }
            if self.craft_import_open {
                ui.group(|ui| {
                    ui.label("Paste preset JSON:");
                    ui.add(egui::TextEdit::multiline(&mut self.craft_import_json)
                        .desired_width(f32::INFINITY)
                        .desired_rows(4));
                    ui.horizontal(|ui| {
                        if ui.button("Import").clicked() {
                            if let Some(preset) = import_preset_json(&self.craft_import_json) {
                                self.preset_manager.add_custom(preset);
                                self.craft_import_json.clear();
                                self.craft_import_open = false;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.craft_import_json.clear();
                            self.craft_import_open = false;
                        }
                    });
                });
            }
        }

        ui.add_space(5.0);

        // --- Harmony ---
        section_header(ui, "Harmony");
        {
            ui.label("Key:");
            egui::ComboBox::from_id_salt("craft_key")
                .selected_text(format!("{}", self.state.key))
                .width(160.0)
                .show_ui(ui, |ui| {
                    for key in Key::all() {
                        ui.selectable_value(&mut self.state.key, *key, format!("{}", key));
                    }
                });
            ui.add_space(5.0);

            ui.label("Mode:");
            egui::ComboBox::from_id_salt("craft_mode")
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
            egui::ComboBox::from_id_salt("craft_octave")
                .selected_text(self.state.octave_mode.description())
                .width(160.0)
                .show_ui(ui, |ui| {
                    for octave_mode in OctaveMode::all() {
                        ui.selectable_value(&mut self.state.octave_mode, *octave_mode, octave_mode.description());
                    }
                });
        }

        ui.add_space(5.0);

        // --- Voice Leading ---
        section_header(ui, "Voice Leading");
        {
            ornate_toggle(ui, "Enable Voice Leading", &mut self.voice_leading_enabled);
            if self.voice_leading_enabled {
                ui.add_space(5.0);
                ui.label("Style:");
                egui::ComboBox::from_id_salt("craft_voice_leading_style")
                    .selected_text(self.voice_leading_style.description())
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        for style in VoiceLeadingStyle::all() {
                            ui.selectable_value(&mut self.voice_leading_style, *style, style.description());
                        }
                    });
            }
        }

        ui.add_space(5.0);

        // --- Metronome ---
        section_header(ui, "Metronome");
        {
            let mut bpm_f32 = self.humanize_config.bpm as f32;
            ornate_slider(ui, "BPM", &mut bpm_f32, 40.0..=240.0);
            self.humanize_config.bpm = bpm_f32 as f64;
            ui.label(format!("Time Sig: {}/{}", self.humanize_config.beats_per_bar, self.humanize_config.beat_unit));
            ornate_toggle(ui, "Metronome Click", &mut self.humanize_config.metronome_enabled);
            if self.humanize_config.metronome_enabled {
                let metro_port = self.humanize_config.metronome_output_port.unwrap_or(0);
                let port_label = self.state.output_slots.get(metro_port)
                    .and_then(|s| *s)
                    .and_then(|idx| self.state.available_outputs.iter().find(|(i, _)| *i == idx))
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| format!("Out {}", metro_port + 1));
                ui.label("Output:");
                egui::ComboBox::from_id_salt("craft_metronome_output")
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
        }

        ui.add_space(5.0);

        // --- Humanization ---
        section_header(ui, "Humanization");
        {
            ornate_toggle(ui, "Enable Humanization", &mut self.humanize_config.enabled);

            if self.humanize_config.enabled {
                ui.add_space(5.0);

                // Timing Jitter
                egui::CollapsingHeader::new("Timing Jitter")
                    .default_open(false)
                    .show(ui, |ui| {
                        ornate_toggle(ui, "Enable Jitter", &mut self.humanize_config.jitter_enabled);
                        if self.humanize_config.jitter_enabled {
                            let mut jmin = self.humanize_config.jitter_min_ms as f32;
                            let mut jmax = self.humanize_config.jitter_max_ms as f32;
                            ornate_slider(ui, "Min ms", &mut jmin, 0.0..=50.0);
                            ornate_slider(ui, "Max ms", &mut jmax, 0.0..=50.0);
                            self.humanize_config.jitter_min_ms = jmin as u16;
                            self.humanize_config.jitter_max_ms = jmax as u16;
                            if self.humanize_config.jitter_min_ms > self.humanize_config.jitter_max_ms {
                                self.humanize_config.jitter_min_ms = self.humanize_config.jitter_max_ms;
                            }
                        }
                    });

                // Velocity
                egui::CollapsingHeader::new("Velocity")
                    .default_open(false)
                    .show(ui, |ui| {
                        ornate_toggle(ui, "Velocity Variation", &mut self.humanize_config.velocity_enabled);
                        if self.humanize_config.velocity_enabled {
                            let mut vel = self.humanize_config.velocity_variation as f32;
                            ornate_slider(ui, "Variation", &mut vel, 0.0..=30.0);
                            self.humanize_config.velocity_variation = vel as u8;
                        }
                    });

                // Duration
                egui::CollapsingHeader::new("Duration")
                    .default_open(false)
                    .show(ui, |ui| {
                        ornate_toggle(ui, "Duration Variation", &mut self.humanize_config.duration_enabled);
                        if self.humanize_config.duration_enabled {
                            let mut dur = self.humanize_config.duration_variation_ms as f32;
                            ornate_slider(ui, "ms", &mut dur, 0.0..=100.0);
                            self.humanize_config.duration_variation_ms = dur as u16;
                        }
                    });

                // Swing/Groove
                egui::CollapsingHeader::new("Swing/Groove")
                    .default_open(false)
                    .show(ui, |ui| {
                        ornate_toggle(ui, "Swing", &mut self.humanize_config.swing_enabled);
                        if self.humanize_config.swing_enabled {
                            ornate_slider(ui, "Amount", &mut self.humanize_config.swing_amount, 0.0..=1.0);
                        }
                    });
            }
        }

        }); // ScrollArea
    }
}
