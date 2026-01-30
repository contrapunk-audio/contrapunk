//! Custom steampunk-themed widgets for Contrapunk.
//!
//! Ornate sliders, toggles, decorative frames, gear helpers, and section headers.

use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Rounding, Sense, Stroke, StrokeKind, Vec2};
use std::ops::RangeInclusive;

use super::colors::*;

/// Ornate slider with gold/copper steampunk styling.
///
/// Draws a labeled horizontal slider with a copper-filled rail and amber knob.
/// Hover brightens knob to gold. Drag updates value clamped to range.
pub fn ornate_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
) -> egui::Response {
    let desired_size = Vec2::new(ui.available_width().min(240.0), 36.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    let label_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width() * 0.5, 14.0));
    painter.text(
        label_rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.0),
        TEXT_SECONDARY,
    );

    let value_text = format!("{:.2}", *value);
    let value_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + rect.width() * 0.5, rect.min.y),
        Vec2::new(rect.width() * 0.5, 14.0),
    );
    painter.text(
        value_rect.right_center(),
        egui::Align2::RIGHT_CENTER,
        &value_text,
        egui::FontId::proportional(11.0),
        TEXT_DIM,
    );

    // Rail area below label
    let rail_y = rect.min.y + 20.0;
    let rail_height = 6.0;
    let rail_rect = Rect::from_min_size(
        Pos2::new(rect.min.x, rail_y),
        Vec2::new(rect.width(), rail_height),
    );

    // Handle drag
    if response.dragged() || response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            let t = ((pointer.x - rail_rect.min.x) / rail_rect.width()).clamp(0.0, 1.0);
            *value = *range.start() + t * (*range.end() - *range.start());
        }
    }

    let t = ((*value - *range.start()) / (*range.end() - *range.start())).clamp(0.0, 1.0);

    // Draw rail background
    painter.rect_filled(rail_rect, Rounding::same(3), WIDGET_INACTIVE);

    // Draw fill
    let fill_rect = Rect::from_min_size(rail_rect.min, Vec2::new(rail_rect.width() * t, rail_height));
    painter.rect_filled(fill_rect, Rounding::same(3), COPPER);

    // Draw knob
    let knob_x = rail_rect.min.x + rail_rect.width() * t;
    let knob_center = Pos2::new(knob_x, rail_y + rail_height * 0.5);
    let knob_radius = 7.0;
    let knob_color = if response.hovered() || response.dragged() {
        GOLD
    } else {
        AMBER
    };
    painter.circle(knob_center, knob_radius, knob_color, Stroke::new(1.5, COPPER_HOVER));

    response
}

/// Ornate pill-shaped toggle with steampunk styling.
///
/// OFF: inactive fill with border stroke. ON: copper fill with gold stroke.
pub fn ornate_toggle(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut bool,
) -> egui::Response {
    let toggle_width = 36.0;
    let toggle_height = 18.0;
    let label_width = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(12.0),
        TEXT_PRIMARY,
    ).rect.width();
    let total_width = toggle_width + 8.0 + label_width;
    let desired_size = Vec2::new(total_width, toggle_height);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
    let painter = ui.painter_at(rect);

    if response.clicked() {
        *value = !*value;
    }

    let pill_rect = Rect::from_min_size(rect.min, Vec2::new(toggle_width, toggle_height));
    let rounding = Rounding::same(toggle_height as u8 / 2);

    let (fill, stroke_color) = if *value {
        (COPPER, GOLD)
    } else {
        (WIDGET_INACTIVE, BORDER)
    };

    painter.rect(pill_rect, rounding, fill, Stroke::new(1.0, stroke_color), StrokeKind::Outside);

    // Circle indicator
    let circle_radius = toggle_height * 0.35;
    let circle_x = if *value {
        pill_rect.max.x - circle_radius - 3.0
    } else {
        pill_rect.min.x + circle_radius + 3.0
    };
    let circle_center = Pos2::new(circle_x, pill_rect.center().y);
    let circle_color = if *value { GOLD } else { TEXT_DIM };
    painter.circle_filled(circle_center, circle_radius, circle_color);

    // Label
    let label_pos = Pos2::new(pill_rect.max.x + 8.0, rect.center().y);
    painter.text(
        label_pos,
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        TEXT_PRIMARY,
    );

    response
}

/// Draw an ornate decorative frame with double border and corner accents.
pub fn draw_ornate_frame(painter: &egui::Painter, rect: Rect) {
    // Outer border
    painter.rect_stroke(rect, Rounding::same(4), Stroke::new(3.0, BORDER), StrokeKind::Outside);

    // Inner border (5px inset)
    let inner = rect.shrink(5.0);
    painter.rect_stroke(inner, Rounding::same(3), Stroke::new(1.0, BORDER), StrokeKind::Outside);

    // Corner accents — small gears at each corner
    let corners = [rect.left_top(), rect.right_top(), rect.left_bottom(), rect.right_bottom()];
    for corner in &corners {
        draw_gear(painter, *corner, 6.0, 8, 0.0, CORNER_ACCENT);
    }
}

/// Draw a gear/cog shape at the given position.
///
/// Draws a filled circle body with radiating teeth (line segments).
pub fn draw_gear(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    teeth: usize,
    angle: f32,
    color: Color32,
) {
    // Body circle
    painter.circle_filled(center, radius * 0.6, color);

    // Teeth as line segments radiating outward
    let tooth_inner = radius * 0.5;
    let tooth_outer = radius;
    for i in 0..teeth {
        let theta = angle + (i as f32) * std::f32::consts::TAU / (teeth as f32);
        let inner_pt = Pos2::new(
            center.x + tooth_inner * theta.cos(),
            center.y + tooth_inner * theta.sin(),
        );
        let outer_pt = Pos2::new(
            center.x + tooth_outer * theta.cos(),
            center.y + tooth_outer * theta.sin(),
        );
        painter.line_segment([inner_pt, outer_pt], Stroke::new(2.0, color));
    }
}

/// Section header with gold text and a horizontal rule below.
pub fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(text).color(GOLD).size(14.0).strong());
    let rect = ui.available_rect_before_wrap();
    let y = rect.min.y;
    let painter = ui.painter();
    painter.hline(rect.min.x..=rect.max.x, y, Stroke::new(1.0, BORDER));
    ui.add_space(4.0);
}
