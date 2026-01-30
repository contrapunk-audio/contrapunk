//! Single-screen three-column layout for the Contrapunk GUI.
//!
//! Three-column layout showing all controls at once — no scrolling, no tabs.

use eframe::egui;
use crate::app::{ContrapunkApp, midi_to_name};
use crate::generator::{GeneratorMode, ArpDirection, ChordType};
use crate::harmony::{Key, HarmonyMode, OctaveMode, VoiceLeadingStyle};
use crate::preset::storage::{export_preset_json, import_preset_json};
use crate::theme::colors::*;
use crate::theme::widgets::{ornate_slider, ornate_toggle};

/// Helper to wrap content in a styled card frame.
fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(WIDGET_BG)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(egui::Rounding::ZERO)
        .inner_margin(egui::Margin::same(6))
        .show(ui, |ui| {
            add_contents(ui);
        });
}

fn card_header(ui: &mut egui::Ui, label: &str) {
    ui.label(egui::RichText::new(label).color(GOLD).size(11.0).strong());
    ui.add_space(2.0);
}

impl ContrapunkApp {
    /// Draws the single-screen three-column layout. No scroll — everything fits.
    pub fn draw_main_ui(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.add_space(2.0);
        ui.columns(3, |cols| {
            self.draw_col_left(&mut cols[0]);
            self.draw_col_center(&mut cols[1]);
            self.draw_col_right(&mut cols[2]);
        });
    }

    /// Left column: Performance, Preset, MIDI I/O.
    fn draw_col_left(&mut self, ui: &mut egui::Ui) {
        // --- Start/Stop ---
        card(ui, |ui| {
            ui.horizontal(|ui| {
                let btn_text = if self.is_running() { "Stop" } else { "Start" };
                let btn_color = if self.is_running() { egui::Color32::from_rgb(255, 0, 77) } else { COPPER };
                if ui.add_sized(
                    [80.0, 24.0],
                    egui::Button::new(
                        egui::RichText::new(btn_text).size(11.0).strong().color(egui::Color32::WHITE)
                    ).fill(btn_color).rounding(egui::Rounding::ZERO)
                ).clicked() {
                    if self.is_running() {
                        self.pending_stop = true;
                    } else {
                        self.pending_start = true;
                    }
                }
                if self.is_running() {
                    ui.label(egui::RichText::new("ACTIVE").size(10.0).color(NOTE_ACTIVE).strong());
                } else {
                    ui.label(egui::RichText::new("STOPPED").size(10.0).color(TEXT_DIM));
                }
            });
        });
        ui.add_space(2.0);

        // --- Preset ---
        card(ui, |ui| {
            card_header(ui, "Preset");
            let active_name = self.preset_manager.active()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "(none)".to_string());
            let preset_info: Vec<(usize, String, bool)> = self.preset_manager.all_presets()
                .iter().enumerate()
                .map(|(i, p)| (i, p.name.clone(), p.name == active_name))
                .collect();
            let mut selected_preset: Option<usize> = None;
            egui::ComboBox::from_id_salt("main_preset_select")
                .selected_text(&active_name)
                .width(ui.available_width() - 8.0)
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
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                if ui.small_button("Save As").clicked() {
                    self.craft_save_as_open = !self.craft_save_as_open;
                }
                if ui.small_button("New").clicked() {
                    let count = self.preset_manager.custom_presets().len() + 1;
                    let name = format!("Custom {}", count);
                    let preset = self.preset_from_current(&name, "Custom", "User");
                    self.preset_manager.add_custom(preset);
                }
                let active_is_custom = self.preset_manager.active()
                    .map(|p| !p.is_builtin)
                    .unwrap_or(false);
                if active_is_custom {
                    if ui.small_button("Del").clicked() {
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
                if ui.small_button("Exp").clicked() {
                    if let Some(preset) = self.preset_manager.active() {
                        self.craft_export_json = export_preset_json(preset);
                    }
                }
                if ui.small_button("Imp").clicked() {
                    self.craft_import_open = !self.craft_import_open;
                }
            });

            if self.craft_save_as_open {
                ui.add_space(2.0);
                ui.group(|ui| {
                    ui.horizontal(|ui| { ui.label("Name:"); ui.text_edit_singleline(&mut self.craft_save_as_name); });
                    ui.horizontal(|ui| { ui.label("Persona:"); ui.text_edit_singleline(&mut self.craft_save_as_persona); });
                    ui.horizontal(|ui| { ui.label("Genre:"); ui.text_edit_singleline(&mut self.craft_save_as_genre); });
                    ui.horizontal(|ui| {
                        if ui.small_button("Save").clicked() && !self.craft_save_as_name.is_empty() {
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
                        if ui.small_button("Cancel").clicked() { self.craft_save_as_open = false; }
                    });
                });
            }

            if !self.craft_export_json.is_empty() {
                ui.add_space(2.0);
                ui.group(|ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.craft_export_json)
                        .desired_width(f32::INFINITY).desired_rows(3));
                    if ui.small_button("Close").clicked() { self.craft_export_json.clear(); }
                });
            }

            if self.craft_import_open {
                ui.add_space(2.0);
                ui.group(|ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.craft_import_json)
                        .desired_width(f32::INFINITY).desired_rows(3));
                    ui.horizontal(|ui| {
                        if ui.small_button("Import").clicked() {
                            if let Some(preset) = import_preset_json(&self.craft_import_json) {
                                self.preset_manager.add_custom(preset);
                                self.craft_import_json.clear();
                                self.craft_import_open = false;
                            }
                        }
                        if ui.small_button("Cancel").clicked() { self.craft_import_json.clear(); self.craft_import_open = false; }
                    });
                });
            }
        });
        ui.add_space(2.0);

        // --- MIDI I/O ---
        card(ui, |ui| {
            card_header(ui, "MIDI I/O");
            // Input
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("In:").color(TEXT_SECONDARY));
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let input_text = match self.state.input_port {
                        Some(idx) => self.state.available_inputs.iter()
                            .find(|(i, _)| *i == idx)
                            .map(|(_, name)| name.clone())
                            .unwrap_or_else(|| format!("Port {}", idx)),
                        None => "Select...".to_string(),
                    };
                    egui::ComboBox::from_id_salt("main_input_port")
                        .selected_text(&input_text)
                        .width(ui.available_width().min(200.0))
                        .show_ui(ui, |ui| {
                            for (idx, name) in &self.state.available_inputs {
                                if ui.selectable_label(self.state.input_port == Some(*idx), name).clicked() {
                                    self.state.input_port = Some(*idx);
                                }
                            }
                        });
                }
                #[cfg(target_arch = "wasm32")]
                {
                    if self.midi_access.borrow().is_none() {
                        ui.colored_label(egui::Color32::YELLOW, "Requesting...");
                    } else if self.state.available_inputs.is_empty() {
                        ui.colored_label(TEXT_DIM, "No devices");
                    } else {
                        let input_text = match self.state.input_port {
                            Some(idx) => self.state.available_inputs.iter()
                                .find(|(i, _)| *i == idx)
                                .map(|(_, name)| name.clone())
                                .unwrap_or_else(|| format!("Port {}", idx)),
                            None => "Select...".to_string(),
                        };
                        egui::ComboBox::from_id_salt("main_input_port")
                            .selected_text(&input_text)
                            .width(ui.available_width().min(200.0))
                            .show_ui(ui, |ui| {
                                for (idx, name) in &self.state.available_inputs {
                                    if ui.selectable_label(self.state.input_port == Some(*idx), name).clicked() {
                                        self.state.input_port = Some(*idx);
                                    }
                                }
                            });
                    }
                }
            });
            // Output slots
            for slot_idx in 0..self.state.output_slots.len() {
                self.draw_output_slot(ui, slot_idx);
            }
            // Refresh
            #[cfg(not(target_arch = "wasm32"))]
            { if ui.small_button("Refresh").clicked() { self.state.refresh_devices(); } }
            #[cfg(target_arch = "wasm32")]
            { if ui.small_button("Refresh").clicked() { self.midi_initialized = false; self.state.refresh_devices(); } }
        });
    }

    /// Center column: Harmony, Voice Leading, Active Notes.
    fn draw_col_center(&mut self, ui: &mut egui::Ui) {
        // --- Harmony ---
        card(ui, |ui| {
            card_header(ui, "Harmony");
            ui.label("Key:");
            egui::ComboBox::from_id_salt("main_key")
                .selected_text(format!("{}", self.state.key))
                .width(80.0)
                .show_ui(ui, |ui| {
                    for key in Key::all() {
                        ui.selectable_value(&mut self.state.key, *key, format!("{}", key));
                    }
                });
            ui.add_space(2.0);
            ui.label("Mode:");
            egui::ComboBox::from_id_salt("main_mode")
                .selected_text(format!("{}: {}", self.state.mode.number(), self.state.mode.description()))
                .width(ui.available_width().min(220.0))
                .show_ui(ui, |ui| {
                    for mode in HarmonyMode::all() {
                        let text = format!("{}: {}", mode.number(), mode.description());
                        ui.selectable_value(&mut self.state.mode, *mode, text);
                    }
                });
            ui.add_space(2.0);
            ui.label("Octave:");
            egui::ComboBox::from_id_salt("main_octave")
                .selected_text(self.state.octave_mode.description())
                .width(ui.available_width().min(180.0))
                .show_ui(ui, |ui| {
                    for octave_mode in OctaveMode::all() {
                        ui.selectable_value(&mut self.state.octave_mode, *octave_mode, octave_mode.description());
                    }
                });
        });
        ui.add_space(2.0);

        // --- Voice Leading ---
        card(ui, |ui| {
            card_header(ui, "Voice Leading");
            ornate_toggle(ui, "Enable", &mut self.voice_leading_enabled);
            if self.voice_leading_enabled {
                ui.add_space(2.0);
                egui::ComboBox::from_id_salt("main_voice_leading_style")
                    .selected_text(self.voice_leading_style.description())
                    .width(ui.available_width().min(180.0))
                    .show_ui(ui, |ui| {
                        for style in VoiceLeadingStyle::all() {
                            ui.selectable_value(&mut self.voice_leading_style, *style, style.description());
                        }
                    });
            }
        });
        ui.add_space(2.0);

        // --- Active Notes ---
        let (input_notes, harmony_notes) = self.get_router_notes();
        card(ui, |ui| {
            card_header(ui, "Active Notes");
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("In:").color(TEXT_SECONDARY));
                if input_notes.is_empty() {
                    ui.label(egui::RichText::new("---").color(TEXT_DIM));
                } else {
                    let mut sorted: Vec<_> = input_notes.iter().copied().collect();
                    sorted.sort();
                    for midi in sorted {
                        ui.label(egui::RichText::new(midi_to_name(midi)).color(NOTE_ACTIVE).strong());
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Harm:").color(TEXT_SECONDARY));
                if harmony_notes.is_empty() {
                    ui.label(egui::RichText::new("---").color(TEXT_DIM));
                } else {
                    let mut sorted: Vec<_> = harmony_notes.iter().copied().collect();
                    sorted.sort();
                    for midi in sorted {
                        ui.label(egui::RichText::new(midi_to_name(midi)).color(HARMONY_NOTE).strong());
                    }
                }
            });
        });

        // Error display
        if let Some(ref error) = self.state.last_error {
            ui.add_space(2.0);
            ui.colored_label(egui::Color32::from_rgb(255, 0, 77), format!("! {}", error));
        }
    }

    /// Right column: Metronome, Humanization.
    fn draw_col_right(&mut self, ui: &mut egui::Ui) {
        // --- Metronome ---
        card(ui, |ui| {
            card_header(ui, "Metronome");
            let mut bpm_f32 = self.humanize_config.bpm as f32;
            ornate_slider(ui, "BPM", &mut bpm_f32, 40.0..=240.0);
            self.humanize_config.bpm = bpm_f32 as f64;
            ornate_toggle(ui, "Click", &mut self.humanize_config.metronome_enabled);
            if self.humanize_config.metronome_enabled {
                let metro_port = self.humanize_config.metronome_output_port.unwrap_or(0);
                let port_label = self.state.output_slots.get(metro_port)
                    .and_then(|s| *s)
                    .and_then(|idx| self.state.available_outputs.iter().find(|(i, _)| *i == idx))
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| format!("Out {}", metro_port + 1));
                ui.horizontal(|ui| {
                    ui.label("Out:");
                    egui::ComboBox::from_id_salt("main_metronome_output")
                        .selected_text(&port_label)
                        .width(120.0)
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
                });
            }
        });
        ui.add_space(2.0);

        // --- Humanization ---
        card(ui, |ui| {
            card_header(ui, "Humanization");
            ornate_toggle(ui, "Enable", &mut self.humanize_config.enabled);

            if self.humanize_config.enabled {
                ui.add_space(2.0);

                ui.label(egui::RichText::new("Jitter").color(TEXT_SECONDARY).strong());
                ornate_toggle(ui, "Enable", &mut self.humanize_config.jitter_enabled);
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

                ui.add_space(2.0);
                ui.label(egui::RichText::new("Velocity").color(TEXT_SECONDARY).strong());
                ornate_toggle(ui, "Enable", &mut self.humanize_config.velocity_enabled);
                if self.humanize_config.velocity_enabled {
                    let mut vel = self.humanize_config.velocity_variation as f32;
                    ornate_slider(ui, "Variation", &mut vel, 0.0..=30.0);
                    self.humanize_config.velocity_variation = vel as u8;
                }

                ui.add_space(2.0);
                ui.label(egui::RichText::new("Duration").color(TEXT_SECONDARY).strong());
                ornate_toggle(ui, "Enable", &mut self.humanize_config.duration_enabled);
                if self.humanize_config.duration_enabled {
                    let mut dur = self.humanize_config.duration_variation_ms as f32;
                    ornate_slider(ui, "ms", &mut dur, 0.0..=100.0);
                    self.humanize_config.duration_variation_ms = dur as u16;
                }

                ui.add_space(2.0);
                ui.label(egui::RichText::new("Swing").color(TEXT_SECONDARY).strong());
                ornate_toggle(ui, "Enable", &mut self.humanize_config.swing_enabled);
                if self.humanize_config.swing_enabled {
                    ornate_slider(ui, "Amount", &mut self.humanize_config.swing_amount, 0.0..=1.0);
                }
            }
        });
        ui.add_space(2.0);

        // --- Note Generator ---
        card(ui, |ui| {
            card_header(ui, "Note Generator");
            ornate_toggle(ui, "Enable", &mut self.generator_enabled);
            let _ = self.generator.set_enabled(self.generator_enabled);

            if self.generator_enabled {
                ui.add_space(2.0);

                // Mode selector
                const MODE_LABELS: &[&str] = &[
                    "Held Notes", "Chord", "Arpeggio Up", "Arpeggio Down",
                    "Arpeggio Up-Down", "Scale Runner", "Random Diatonic",
                ];
                let prev_mode_idx = self.generator_mode_index;
                egui::ComboBox::from_id_salt("gen_mode")
                    .selected_text(MODE_LABELS[self.generator_mode_index])
                    .width(ui.available_width().min(180.0))
                    .show_ui(ui, |ui| {
                        for (i, label) in MODE_LABELS.iter().enumerate() {
                            ui.selectable_value(&mut self.generator_mode_index, i, *label);
                        }
                    });
                if self.generator_mode_index != prev_mode_idx {
                    let mode = match self.generator_mode_index {
                        0 => GeneratorMode::HeldNotes,
                        1 => GeneratorMode::Chord(Self::chord_type_from_index(self.generator_chord_quality)),
                        2 => GeneratorMode::Arpeggio(ArpDirection::Up),
                        3 => GeneratorMode::Arpeggio(ArpDirection::Down),
                        4 => GeneratorMode::Arpeggio(ArpDirection::UpDown),
                        5 => GeneratorMode::ScaleRunner,
                        _ => GeneratorMode::RandomDiatonic,
                    };
                    let _events = self.generator.set_mode(mode);
                }

                // Chord picker (visible only in Chord mode)
                if self.generator_mode_index == 1 {
                    ui.add_space(2.0);
                    const ROOT_LABELS: &[&str] = &[
                        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
                    ];
                    const QUALITY_LABELS: &[&str] = &[
                        "Major", "Minor", "Dim", "Aug", "Maj7", "Min7", "Dom7", "Dim7", "HalfDim7",
                    ];
                    let prev_root = self.generator_chord_root;
                    let prev_qual = self.generator_chord_quality;
                    ui.horizontal(|ui| {
                        ui.label("Root:");
                        egui::ComboBox::from_id_salt("gen_root")
                            .selected_text(ROOT_LABELS[self.generator_chord_root])
                            .width(50.0)
                            .show_ui(ui, |ui| {
                                for (i, label) in ROOT_LABELS.iter().enumerate() {
                                    ui.selectable_value(&mut self.generator_chord_root, i, *label);
                                }
                            });
                        ui.label("Quality:");
                        egui::ComboBox::from_id_salt("gen_quality")
                            .selected_text(QUALITY_LABELS[self.generator_chord_quality])
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                for (i, label) in QUALITY_LABELS.iter().enumerate() {
                                    ui.selectable_value(&mut self.generator_chord_quality, i, *label);
                                }
                            });
                    });
                    // Resolve chord to MIDI notes when changed
                    if self.generator_chord_root != prev_root || self.generator_chord_quality != prev_qual {
                        let chord_type = Self::chord_type_from_index(self.generator_chord_quality);
                        let root_midi = 60u8 + self.generator_chord_root as u8; // C4 = 60
                        let notes: Vec<wmidi::Note> = chord_type.intervals().iter()
                            .map(|&interval| wmidi::Note::from_u8_lossy(root_midi + interval))
                            .collect();
                        self.generator_selected_notes = notes.iter().map(|n| *n as u8).collect();
                        let _events = self.generator.set_selected_notes(notes);
                        // Also update mode to new chord type
                        let _events = self.generator.set_mode(GeneratorMode::Chord(chord_type));
                    }
                }

                ui.add_space(2.0);

                // Duration slider
                let mut dur = self.generator.note_duration_beats() as f32;
                ornate_slider(ui, "Duration (beats)", &mut dur, 0.125..=2.0);
                // Snap to 0.125 steps
                dur = (dur / 0.125).round() * 0.125;
                self.generator.set_note_duration_beats(dur as f64);

                // Velocity slider
                let mut vel = self.generator.velocity() as f32;
                ornate_slider(ui, "Velocity", &mut vel, 1.0..=127.0);
                self.generator.set_velocity(vel as u8);

                ui.add_space(2.0);

                // Selected notes display
                let sel_text = if self.generator_selected_notes.is_empty() {
                    "No notes selected".to_string()
                } else {
                    self.generator_selected_notes.iter()
                        .map(|&n| midi_to_name(n))
                        .collect::<Vec<_>>()
                        .join(" ")
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Notes:").color(TEXT_SECONDARY));
                    ui.label(egui::RichText::new(sel_text).color(egui::Color32::from_rgb(0, 220, 220)));
                });
            }
        });
    }

    fn chord_type_from_index(idx: usize) -> ChordType {
        match idx {
            0 => ChordType::Major,
            1 => ChordType::Minor,
            2 => ChordType::Dim,
            3 => ChordType::Aug,
            4 => ChordType::Maj7,
            5 => ChordType::Min7,
            6 => ChordType::Dom7,
            7 => ChordType::Dim7,
            _ => ChordType::HalfDim7,
        }
    }

    fn draw_output_slot(&mut self, ui: &mut egui::Ui, slot_idx: usize) {
        let output_text = match self.state.output_slots[slot_idx] {
            Some(idx) => self.state.available_outputs.iter()
                .find(|(i, _)| *i == idx)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| format!("Port {}", idx)),
            None => "None".to_string(),
        };
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{}", slot_idx + 1)).color(TEXT_DIM));
            egui::ComboBox::from_id_salt(format!("main_output_slot_{}", slot_idx))
                .selected_text(&output_text)
                .width(ui.available_width().min(200.0))
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
        });
    }
}
