//! Steampunk color palette for Contrapunk.
//!
//! Named Color32 constants for the dark steampunk theme.

use eframe::egui::Color32;

// --- Background colors ---
pub const DARK_BG: Color32 = Color32::from_rgb(15, 12, 8);
pub const PANEL_BG: Color32 = Color32::from_rgb(20, 16, 12);
pub const WIDGET_BG: Color32 = Color32::from_rgb(25, 20, 15);
pub const WIDGET_INACTIVE: Color32 = Color32::from_rgb(45, 35, 25);

// --- Accent colors ---
pub const GOLD: Color32 = Color32::from_rgb(255, 215, 140);
pub const COPPER: Color32 = Color32::from_rgb(180, 120, 50);
pub const AMBER: Color32 = Color32::from_rgb(220, 170, 60);
pub const COPPER_HOVER: Color32 = Color32::from_rgb(120, 85, 40);

// --- Border colors ---
pub const BORDER: Color32 = Color32::from_rgb(160, 110, 40);
pub const CORNER_ACCENT: Color32 = Color32::from_rgb(200, 150, 60);

// --- Text colors ---
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 215, 140);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 160, 130);
pub const TEXT_DIM: Color32 = Color32::from_rgb(120, 100, 75);

// --- Special colors ---
pub const NOTE_ACTIVE: Color32 = Color32::from_rgb(100, 220, 100);
pub const HARMONY_NOTE: Color32 = Color32::from_rgb(100, 150, 255);
