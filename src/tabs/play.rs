//! Play tab: performance-focused controls.
//!
//! Shows key, mode, MIDI device selection, start/stop, active preset quick-switch,
//! and active notes display.

use eframe::egui;
use crate::app::{ContrapunkApp, midi_to_name};
use crate::harmony::{Key, HarmonyMode};
use crate::theme::colors::*;

impl ContrapunkApp {
    /// Draws the Play tab content: simplified performance controls.
    pub fn draw_play_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.add_space(12.0);

            // Center content with max width
            let max_w = 700.0_f32.min(ui.available_width());
            ui.allocate_ui_with_layout(
                egui::vec2(max_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.set_max_width(max_w);

                    // --- Start/Stop card ---
                    egui::Frame::group(ui.style())
                        .fill(WIDGET_BG)
                        .stroke(egui::Stroke::new(1.0, BORDER))
                        .rounding(egui::Rounding::same(8))
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let btn_text = if self.is_running() { "Stop" } else { "Start" };
                                let btn_color = if self.is_running() { egui::Color32::from_rgb(180, 60, 60) } else { COPPER };
                                if ui.add_sized(
                                    [140.0, 45.0],
                                    egui::Button::new(
                                        egui::RichText::new(btn_text).size(18.0).strong().color(egui::Color32::WHITE)
                                    ).fill(btn_color).rounding(egui::Rounding::same(6))
                                ).clicked() {
                                    if self.is_running() {
                                        self.pending_stop = true;
                                    } else {
                                        self.pending_start = true;
                                    }
                                }

                                ui.add_space(20.0);
                                if self.is_running() {
                                    ui.label(egui::RichText::new("● ACTIVE").size(16.0).color(NOTE_ACTIVE).strong());
                                } else {
                                    ui.label(egui::RichText::new("○ STOPPED").size(16.0).color(TEXT_DIM));
                                }
                            });
                        });

                    ui.add_space(12.0);

                    // --- Preset card ---
                    egui::Frame::group(ui.style())
                        .fill(WIDGET_BG)
                        .stroke(egui::Stroke::new(1.0, BORDER))
                        .rounding(egui::Rounding::same(8))
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Preset").color(GOLD).size(14.0).strong());
                            ui.add_space(4.0);

                            if let Some(preset) = self.preset_manager.active() {
                                ui.label(
                                    egui::RichText::new(&preset.persona)
                                        .size(20.0).strong().color(GOLD)
                                );
                                ui.label(
                                    egui::RichText::new(&preset.genre)
                                        .size(12.0).italics().color(TEXT_DIM)
                                );
                            }

                            ui.add_space(6.0);
                            let active_name = self.preset_manager.active()
                                .map(|p| p.name.clone())
                                .unwrap_or_else(|| "(none)".to_string());
                            let preset_info: Vec<(usize, String, bool)> = self.preset_manager.all_presets()
                                .iter().enumerate()
                                .map(|(i, p)| (i, format!("{} — {}", p.name, p.genre), p.name == active_name))
                                .collect();
                            let mut selected_preset: Option<usize> = None;
                            egui::ComboBox::from_id_salt("play_preset_select")
                                .selected_text(&active_name)
                                .width(ui.available_width().min(400.0))
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

                    ui.add_space(12.0);

                    // --- Harmony card ---
                    egui::Frame::group(ui.style())
                        .fill(WIDGET_BG)
                        .stroke(egui::Stroke::new(1.0, BORDER))
                        .rounding(egui::Rounding::same(8))
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Harmony").color(GOLD).size(14.0).strong());
                            ui.add_space(6.0);

                            ui.columns(2, |cols| {
                                cols[0].label("Key:");
                                egui::ComboBox::from_id_salt("play_key")
                                    .selected_text(format!("{}", self.state.key))
                                    .width(120.0)
                                    .show_ui(&mut cols[0], |ui| {
                                        for key in Key::all() {
                                            ui.selectable_value(&mut self.state.key, *key, format!("{}", key));
                                        }
                                    });

                                cols[1].label("Mode:");
                                egui::ComboBox::from_id_salt("play_mode")
                                    .selected_text(format!("{}", self.state.mode.description()))
                                    .width(200.0)
                                    .show_ui(&mut cols[1], |ui| {
                                        for mode in HarmonyMode::all() {
                                            let text = format!("{}: {}", mode.number(), mode.description());
                                            ui.selectable_value(&mut self.state.mode, *mode, text);
                                        }
                                    });
                            });
                        });

                    ui.add_space(12.0);

                    // --- MIDI Input card ---
                    egui::Frame::group(ui.style())
                        .fill(WIDGET_BG)
                        .stroke(egui::Stroke::new(1.0, BORDER))
                        .rounding(egui::Rounding::same(8))
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("MIDI Input").color(GOLD).size(14.0).strong());
                            ui.add_space(4.0);

                            #[cfg(not(target_arch = "wasm32"))]
                            {
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
                                    .width(ui.available_width().min(400.0))
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
                                if self.midi_access.borrow().is_none() {
                                    ui.colored_label(egui::Color32::YELLOW, "Requesting MIDI access...");
                                } else if self.state.available_inputs.is_empty() {
                                    ui.colored_label(TEXT_DIM, "No MIDI devices found. Connect a device and refresh.");
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
                                        .width(ui.available_width().min(400.0))
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
                        });

                    ui.add_space(12.0);

                    // --- Active Notes card ---
                    let (input_notes, harmony_notes) = self.get_router_notes();
                    egui::Frame::group(ui.style())
                        .fill(WIDGET_BG)
                        .stroke(egui::Stroke::new(1.0, BORDER))
                        .rounding(egui::Rounding::same(8))
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Active Notes").color(GOLD).size(14.0).strong());
                            ui.add_space(4.0);

                            ui.horizontal_wrapped(|ui| {
                                ui.label(egui::RichText::new("Input:").color(TEXT_SECONDARY));
                                if input_notes.is_empty() {
                                    ui.label(egui::RichText::new("—").color(TEXT_DIM));
                                } else {
                                    let mut sorted: Vec<_> = input_notes.iter().copied().collect();
                                    sorted.sort();
                                    for midi in sorted {
                                        ui.label(
                                            egui::RichText::new(midi_to_name(midi))
                                                .color(NOTE_ACTIVE).strong()
                                        );
                                    }
                                }
                            });

                            ui.horizontal_wrapped(|ui| {
                                ui.label(egui::RichText::new("Harmony:").color(TEXT_SECONDARY));
                                if harmony_notes.is_empty() {
                                    ui.label(egui::RichText::new("—").color(TEXT_DIM));
                                } else {
                                    let mut sorted: Vec<_> = harmony_notes.iter().copied().collect();
                                    sorted.sort();
                                    for midi in sorted {
                                        ui.label(
                                            egui::RichText::new(midi_to_name(midi))
                                                .color(HARMONY_NOTE).strong()
                                        );
                                    }
                                }
                            });
                        });

                    // Error display
                    if let Some(ref error) = self.state.last_error {
                        ui.add_space(8.0);
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                    }
                },
            );
        });
    }
}
