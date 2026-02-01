//! Retro pixel art color palette for Contrapunk.
//!
//! Based on the PICO-8 palette with extensions for UI needs.
//! Every color is chosen for pixel-art coherence.

use eframe::egui::Color32;

// --- Background colors (dark layers) ---
pub const DARK_BG: Color32 = Color32::from_rgb(0, 0, 0);          // #000000 PICO-8 black
pub const PANEL_BG: Color32 = Color32::from_rgb(17, 14, 27);      // very dark purple
pub const WIDGET_BG: Color32 = Color32::from_rgb(29, 24, 43);     // dark indigo
pub const WIDGET_INACTIVE: Color32 = Color32::from_rgb(41, 36, 58); // muted purple

// --- Accent colors (PICO-8 inspired) ---
pub const GOLD: Color32 = Color32::from_rgb(0, 228, 54);          // PICO-8 green — primary accent
pub const COPPER: Color32 = Color32::from_rgb(0, 180, 42);        // darker green — buttons/fills
pub const AMBER: Color32 = Color32::from_rgb(0, 200, 48);         // mid green — knobs
pub const COPPER_HOVER: Color32 = Color32::from_rgb(0, 135, 81);  // teal-green hover

// --- Highlight / secondary accent ---
pub const PINK: Color32 = Color32::from_rgb(255, 119, 168);       // PICO-8 pink
pub const CYAN: Color32 = Color32::from_rgb(41, 173, 255);        // PICO-8 blue
pub const YELLOW: Color32 = Color32::from_rgb(255, 236, 39);      // PICO-8 yellow

// --- Border colors ---
pub const BORDER: Color32 = Color32::from_rgb(63, 56, 89);        // muted purple border
pub const CORNER_ACCENT: Color32 = Color32::from_rgb(0, 228, 54); // green corners

// --- Text colors ---
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 241, 232);  // PICO-8 light peach/white
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(194, 195, 199); // PICO-8 light gray
pub const TEXT_DIM: Color32 = Color32::from_rgb(95, 87, 79);         // PICO-8 dark gray

// --- Special colors ---
pub const NOTE_ACTIVE: Color32 = Color32::from_rgb(0, 228, 54);    // PICO-8 green
pub const HARMONY_NOTE: Color32 = Color32::from_rgb(41, 173, 255); // PICO-8 blue

// --- Extra PICO-8 palette for decorative use ---
pub const PICO_RED: Color32 = Color32::from_rgb(255, 0, 77);
pub const PICO_ORANGE: Color32 = Color32::from_rgb(255, 163, 0);
pub const PICO_INDIGO: Color32 = Color32::from_rgb(83, 60, 135);
pub const PICO_DARK_PURPLE: Color32 = Color32::from_rgb(29, 24, 43);
pub const PICO_DARK_GREEN: Color32 = Color32::from_rgb(0, 135, 81);
pub const PICO_BROWN: Color32 = Color32::from_rgb(171, 82, 54);
