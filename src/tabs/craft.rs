//! Craft tab: full parameter editor for harmony, humanization, and voice leading.
//!
//! Combines all parameter controls in one deep-editing workspace.

use eframe::egui;
use crate::app::ContrapunkApp;
use crate::harmony::{Key, HarmonyMode, OctaveMode, VoiceLeadingStyle};

impl ContrapunkApp {
    /// Draws the Craft tab content: full parameter editor.
    pub fn draw_craft_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Craft");
        ui.add_space(8.0);

        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {

        // --- Preset Management ---
        egui::CollapsingHeader::new(egui::RichText::new("Preset").strong())
            .default_open(true)
            .show(ui, |ui| {
                let active_display = self.preset_manager.active()
                    .map(|p| format!("{} ({})", p.name, p.genre))
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
            });

        ui.add_space(5.0);

        // --- Harmony ---
        egui::CollapsingHeader::new(egui::RichText::new("Harmony").strong())
            .default_open(true)
            .show(ui, |ui| {
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
            });

        ui.add_space(5.0);

        // --- Voice Leading ---
        egui::CollapsingHeader::new(egui::RichText::new("Voice Leading").strong())
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(&mut self.voice_leading_enabled, "Enable Voice Leading");
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
            });

        ui.add_space(5.0);

        // --- Metronome ---
        egui::CollapsingHeader::new(egui::RichText::new("Metronome").strong())
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
            });

        ui.add_space(5.0);

        // --- Humanization ---
        egui::CollapsingHeader::new(egui::RichText::new("Humanization").strong())
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

        }); // ScrollArea
    }
}
