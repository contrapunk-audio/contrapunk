//! Retro pixel-art styled widgets for Contrapunk.
//!
//! Sharp corners, stepped borders, pixel-grid knobs, CRT scanline overlays.
//! All rendering snaps to a virtual pixel grid for authentic retro feel.

use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Rounding, Sense, Stroke, StrokeKind, Vec2};
use std::ops::RangeInclusive;

use super::colors::*;

/// Virtual pixel size — all decorative elements snap to this grid.
const PX: f32 = 2.0;

// ─── Pixel-art drawing primitives ────────────────────────────────────

/// Draw a single "pixel" (PX×PX filled rect) at grid position.
fn draw_pixel(painter: &egui::Painter, x: f32, y: f32, color: Color32) {
    painter.rect_filled(
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(PX, PX)),
        Rounding::ZERO,
        color,
    );
}

/// Draw a pixel-art border around a rect — stepped corners, no rounding.
/// Creates an 8-bit style frame with notched corners.
pub fn draw_pixel_border(painter: &egui::Painter, rect: Rect, color: Color32) {
    let l = rect.min.x;
    let r = rect.max.x;
    let t = rect.min.y;
    let b = rect.max.y;

    // Top edge (skip corners)
    painter.line_segment(
        [Pos2::new(l + PX * 2.0, t), Pos2::new(r - PX * 2.0, t)],
        Stroke::new(PX, color),
    );
    // Bottom edge
    painter.line_segment(
        [Pos2::new(l + PX * 2.0, b), Pos2::new(r - PX * 2.0, b)],
        Stroke::new(PX, color),
    );
    // Left edge
    painter.line_segment(
        [Pos2::new(l, t + PX * 2.0), Pos2::new(l, b - PX * 2.0)],
        Stroke::new(PX, color),
    );
    // Right edge
    painter.line_segment(
        [Pos2::new(r, t + PX * 2.0), Pos2::new(r, b - PX * 2.0)],
        Stroke::new(PX, color),
    );

    // Stepped corners (2-pixel steps for 8-bit look)
    // Top-left
    draw_pixel(painter, l, t + PX, color);
    draw_pixel(painter, l + PX, t, color);
    // Top-right
    draw_pixel(painter, r - PX * 2.0, t, color);
    draw_pixel(painter, r - PX, t + PX, color);
    // Bottom-left
    draw_pixel(painter, l, b - PX * 2.0, color);
    draw_pixel(painter, l + PX, b - PX, color);
    // Bottom-right
    draw_pixel(painter, r - PX * 2.0, b - PX, color);
    draw_pixel(painter, r - PX, b - PX * 2.0, color);
}

/// Draw CRT scanline overlay across the entire screen.
/// Horizontal lines every 3px at low opacity for authentic CRT feel.
pub fn draw_scanlines(painter: &egui::Painter, rect: Rect) {
    let scanline_color = Color32::from_rgba_premultiplied(0, 0, 0, 18);
    let mut y = rect.min.y;
    while y < rect.max.y {
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0, scanline_color),
        );
        y += 3.0;
    }
}

/// Draw a pixel-art "CONTRAPUNK" title banner using block letters.
/// Each character is drawn on a 5×5 pixel grid.
pub fn draw_pixel_title(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Simplified block-letter rendering for "CONTRAPUNK"
    // Each letter is 4px wide, 5px tall, 1px gap between letters
    // We'll draw a stylized header bar instead of individual letters
    let title_w = 120.0;
    let title_h = PX * 3.0;
    let start_x = center.x - title_w * 0.5;
    let y = center.y - title_h * 0.5;

    // Decorative bar with notches
    let bar = Rect::from_min_size(
        Pos2::new(start_x, y),
        Vec2::new(title_w, title_h),
    );
    painter.rect_filled(bar, Rounding::ZERO, color);

    // Notch pattern (every 8px, skip a pixel for retro feel)
    let mut nx = start_x;
    while nx < start_x + title_w {
        draw_pixel(painter, nx, y, DARK_BG);
        draw_pixel(painter, nx, y + title_h - PX, DARK_BG);
        nx += PX * 4.0;
    }
}

// ─── Pixel-art UI widgets ────────────────────────────────────────────

/// Retro pixel-art slider with stepped rail and square knob.
pub fn ornate_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
) -> egui::Response {
    let desired_size = Vec2::new(ui.available_width().min(280.0), 36.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    // Label
    painter.text(
        Pos2::new(rect.min.x, rect.min.y + 2.0),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::monospace(11.0),
        TEXT_SECONDARY,
    );

    // Value text
    let value_text = format!("{:.1}", *value);
    painter.text(
        Pos2::new(rect.max.x, rect.min.y + 2.0),
        egui::Align2::RIGHT_TOP,
        &value_text,
        egui::FontId::monospace(11.0),
        GOLD,
    );

    // Rail area
    let rail_y = rect.min.y + 20.0;
    let rail_height = PX * 3.0;
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

    // Draw rail background — sharp rect, no rounding
    painter.rect_filled(rail_rect, Rounding::ZERO, WIDGET_INACTIVE);

    // Draw fill — pixel-snapped
    let fill_w = (rail_rect.width() * t).max(0.0);
    let fill_w_snapped = (fill_w / PX).floor() * PX;
    if fill_w_snapped > 0.0 {
        let fill_rect = Rect::from_min_size(rail_rect.min, Vec2::new(fill_w_snapped, rail_height));
        painter.rect_filled(fill_rect, Rounding::ZERO, COPPER);
    }

    // Draw knob — square, pixel-art style
    let knob_x = rail_rect.min.x + fill_w_snapped;
    let knob_size = PX * 5.0;
    let knob_rect = Rect::from_center_size(
        Pos2::new(knob_x, rail_y + rail_height * 0.5),
        Vec2::new(knob_size, knob_size),
    );
    let knob_color = if response.hovered() || response.dragged() { GOLD } else { AMBER };
    painter.rect_filled(knob_rect, Rounding::ZERO, knob_color);
    // Knob highlight pixel (top-left)
    draw_pixel(&painter, knob_rect.min.x, knob_rect.min.y, Color32::WHITE);

    response
}

/// Retro pixel-art toggle — square pill with sliding block indicator.
pub fn ornate_toggle(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut bool,
) -> egui::Response {
    let toggle_width = PX * 16.0;
    let toggle_height = PX * 8.0;
    let label_width = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::monospace(12.0),
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

    // Background — sharp rect
    let (fill, border_color) = if *value {
        (COPPER, GOLD)
    } else {
        (WIDGET_INACTIVE, BORDER)
    };
    painter.rect_filled(pill_rect, Rounding::ZERO, fill);
    painter.rect_stroke(pill_rect, Rounding::ZERO, Stroke::new(PX, border_color), StrokeKind::Outside);

    // Square indicator block
    let block_size = PX * 5.0;
    let block_x = if *value {
        pill_rect.max.x - block_size - PX * 2.0
    } else {
        pill_rect.min.x + PX * 2.0
    };
    let block_y = pill_rect.min.y + (toggle_height - block_size) * 0.5;
    let block_rect = Rect::from_min_size(Pos2::new(block_x, block_y), Vec2::new(block_size, block_size));
    let block_color = if *value { GOLD } else { TEXT_DIM };
    painter.rect_filled(block_rect, Rounding::ZERO, block_color);
    // Highlight pixel
    if *value {
        draw_pixel(&painter, block_rect.min.x, block_rect.min.y, Color32::WHITE);
    }

    // Label — monospace for retro feel
    let label_pos = Pos2::new(pill_rect.max.x + 8.0, rect.center().y);
    painter.text(
        label_pos,
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::monospace(12.0),
        TEXT_PRIMARY,
    );

    response
}

/// Draw a pixel-art decorative frame (replaces ornate_frame).
/// Double-line border with corner notches and optional inner glow.
pub fn draw_ornate_frame(painter: &egui::Painter, rect: Rect) {
    // Outer pixel border
    draw_pixel_border(painter, rect, BORDER);

    // Inner border (inset by 4px)
    let inner = rect.shrink(PX * 3.0);
    draw_pixel_border(painter, inner, PICO_INDIGO);

    // Corner diamonds — 3×3 pixel diamonds at each corner
    let corners = [
        rect.left_top(),
        Pos2::new(rect.max.x - PX * 3.0, rect.min.y),
        Pos2::new(rect.min.x, rect.max.y - PX * 3.0),
        Pos2::new(rect.max.x - PX * 3.0, rect.max.y - PX * 3.0),
    ];
    for c in &corners {
        // 3-pixel diamond shape
        draw_pixel(painter, c.x + PX, c.y, CORNER_ACCENT);
        draw_pixel(painter, c.x, c.y + PX, CORNER_ACCENT);
        draw_pixel(painter, c.x + PX, c.y + PX, CORNER_ACCENT);
        draw_pixel(painter, c.x + PX * 2.0, c.y + PX, CORNER_ACCENT);
        draw_pixel(painter, c.x + PX, c.y + PX * 2.0, CORNER_ACCENT);
    }
}

/// Draw a pixel-art gear/cog (simplified to a plus/cross shape for pixel aesthetic).
pub fn draw_gear(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    _teeth: usize,
    _angle: f32,
    color: Color32,
) {
    // Pixel cross shape instead of smooth gear
    let r = (radius / PX).floor() * PX;
    // Vertical bar
    painter.rect_filled(
        Rect::from_center_size(center, Vec2::new(PX * 2.0, r * 2.0)),
        Rounding::ZERO,
        color,
    );
    // Horizontal bar
    painter.rect_filled(
        Rect::from_center_size(center, Vec2::new(r * 2.0, PX * 2.0)),
        Rounding::ZERO,
        color,
    );
    // Center pixel brighter
    draw_pixel(painter, center.x - PX * 0.5, center.y - PX * 0.5, Color32::WHITE);
}

/// Section header — retro style with pixel underline.
pub fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(text).color(GOLD).size(14.0).strong());
    let rect = ui.available_rect_before_wrap();
    let y = rect.min.y;
    let painter = ui.painter();
    // Dashed pixel line
    let mut x = rect.min.x;
    while x < rect.max.x {
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(PX * 3.0, PX)),
            Rounding::ZERO,
            BORDER,
        );
        x += PX * 5.0;
    }
    ui.add_space(4.0);
}
