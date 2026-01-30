//! Steampunk theme system for Contrapunk.
//!
//! Provides a dark steampunk color scheme applied globally via egui Visuals.

pub mod colors;
pub mod widgets;

use eframe::egui;
use colors::*;

/// Contrapunk steampunk theme.
pub struct ContrapunkTheme;

impl ContrapunkTheme {
    /// Apply the steampunk theme to the given egui context.
    ///
    /// Constructs dark visuals and overrides widget states, panel fills,
    /// text color, selection, and window rounding with steampunk palette.
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Panel and window backgrounds
        visuals.panel_fill = PANEL_BG;
        visuals.window_fill = PANEL_BG;
        visuals.extreme_bg_color = DARK_BG;
        visuals.faint_bg_color = WIDGET_BG;

        // Override text color globally
        visuals.override_text_color = Some(TEXT_PRIMARY);

        // Selection
        visuals.selection.bg_fill = COPPER;
        visuals.selection.stroke = egui::Stroke::new(1.0, GOLD);

        // Window rounding
        visuals.window_corner_radius = egui::Rounding::same(4);

        // Widget states
        // Noninteractive (labels, static text)
        visuals.widgets.noninteractive.bg_fill = PANEL_BG;
        visuals.widgets.noninteractive.weak_bg_fill = PANEL_BG;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
        visuals.widgets.noninteractive.corner_radius = egui::Rounding::same(3);

        // Inactive (buttons, sliders at rest)
        visuals.widgets.inactive.bg_fill = WIDGET_INACTIVE;
        visuals.widgets.inactive.weak_bg_fill = WIDGET_INACTIVE;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
        visuals.widgets.inactive.corner_radius = egui::Rounding::same(3);

        // Hovered
        visuals.widgets.hovered.bg_fill = COPPER_HOVER;
        visuals.widgets.hovered.weak_bg_fill = COPPER_HOVER;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, CORNER_ACCENT);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, GOLD);
        visuals.widgets.hovered.corner_radius = egui::Rounding::same(3);

        // Active (pressed/clicked)
        visuals.widgets.active.bg_fill = COPPER;
        visuals.widgets.active.weak_bg_fill = COPPER;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, GOLD);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, GOLD);
        visuals.widgets.active.corner_radius = egui::Rounding::same(3);

        // Open (expanded combo boxes, etc.)
        visuals.widgets.open.bg_fill = WIDGET_BG;
        visuals.widgets.open.weak_bg_fill = WIDGET_BG;
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, AMBER);
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, GOLD);
        visuals.widgets.open.corner_radius = egui::Rounding::same(3);

        ctx.set_visuals(visuals);
    }
}
