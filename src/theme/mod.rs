//! Retro pixel-art theme system for Contrapunk.
//!
//! PICO-8 inspired palette, sharp corners, pixel font. Full 8-bit aesthetic.
//! Uses Press Start 2P (OFL license) for authentic retro typography.

pub mod colors;
pub mod widgets;

use eframe::egui;
use colors::*;

/// Embedded Press Start 2P pixel font (OFL licensed, Google Fonts).
const PRESS_START_2P: &[u8] = include_bytes!("../../assets/fonts/PressStart2P-Regular.ttf");

/// Contrapunk retro pixel theme.
pub struct ContrapunkTheme;

impl ContrapunkTheme {
    /// Apply the retro pixel theme to the given egui context.
    pub fn apply(ctx: &egui::Context) {
        // --- Font configuration ---
        let mut fonts = egui::FontDefinitions::default();

        // Register Press Start 2P
        fonts.font_data.insert(
            "PressStart2P".to_owned(),
            egui::FontData::from_static(PRESS_START_2P).into(),
        );

        // Set as primary font for both Proportional and Monospace families
        fonts.families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "PressStart2P".to_owned());

        fonts.families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "PressStart2P".to_owned());

        ctx.set_fonts(fonts);

        // --- Text style sizes ---
        use egui::{FontId, FontFamily, TextStyle};
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Small,    FontId::new(8.0,  FontFamily::Proportional)),
            (TextStyle::Body,     FontId::new(10.0, FontFamily::Proportional)),
            (TextStyle::Button,   FontId::new(10.0, FontFamily::Proportional)),
            (TextStyle::Heading,  FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Monospace,FontId::new(10.0, FontFamily::Monospace)),
        ].into();

        // Increase widget spacing for pixel font readability
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);

        ctx.set_style(style);

        // --- Visuals ---
        let mut visuals = egui::Visuals::dark();

        // Panel and window backgrounds — dark purple base
        visuals.panel_fill = PANEL_BG;
        visuals.window_fill = WIDGET_BG;
        visuals.extreme_bg_color = DARK_BG;
        visuals.faint_bg_color = WIDGET_BG;

        // Text — PICO-8 light peach
        visuals.override_text_color = Some(TEXT_PRIMARY);

        // Selection — green highlight
        visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(0, 180, 42, 80);
        visuals.selection.stroke = egui::Stroke::new(1.0, GOLD);

        // Window rounding — ZERO for pixel art
        visuals.window_corner_radius = egui::Rounding::ZERO;

        // Widget states — all sharp corners
        visuals.widgets.noninteractive.bg_fill = PANEL_BG;
        visuals.widgets.noninteractive.weak_bg_fill = PANEL_BG;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
        visuals.widgets.noninteractive.corner_radius = egui::Rounding::ZERO;

        visuals.widgets.inactive.bg_fill = WIDGET_INACTIVE;
        visuals.widgets.inactive.weak_bg_fill = WIDGET_INACTIVE;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
        visuals.widgets.inactive.corner_radius = egui::Rounding::ZERO;

        visuals.widgets.hovered.bg_fill = PICO_INDIGO;
        visuals.widgets.hovered.weak_bg_fill = PICO_INDIGO;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, GOLD);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(2.0, GOLD);
        visuals.widgets.hovered.corner_radius = egui::Rounding::ZERO;

        visuals.widgets.active.bg_fill = COPPER;
        visuals.widgets.active.weak_bg_fill = COPPER;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, GOLD);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
        visuals.widgets.active.corner_radius = egui::Rounding::ZERO;

        visuals.widgets.open.bg_fill = WIDGET_BG;
        visuals.widgets.open.weak_bg_fill = WIDGET_BG;
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, GOLD);
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, GOLD);
        visuals.widgets.open.corner_radius = egui::Rounding::ZERO;

        ctx.set_visuals(visuals);
    }
}
