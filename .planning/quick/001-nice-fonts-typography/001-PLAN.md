# Quick Task 001: Nice Fonts Typography

## Goal
Embed a high-quality pixel font (Press Start 2P) for the retro PICO-8 aesthetic and configure consistent typography across the entire GUI.

## Tasks

### Task 1: Download and embed Press Start 2P font
- Download PressStart2P-Regular.ttf (OFL license, Google Fonts)
- Create `assets/fonts/` directory
- Include font bytes via `include_bytes!()` at compile time

### Task 2: Configure FontDefinitions in theme
- In `ContrapunkTheme::apply()`, set up `FontDefinitions`
- Register Press Start 2P as primary Proportional font
- Keep system monospace as fallback
- Set default body text size to 13px, heading sizes appropriately
- Configure `TextStyle` mappings for Body, Button, Heading, Small, Monospace

### Task 3: Clean up per-widget font overrides
- Remove excessive `.family(egui::FontFamily::Monospace)` annotations in `ui.rs` and `app.rs`
- Since the theme font is already pixel-styled, most widgets can use the default
- Keep Monospace only where fixed-width alignment matters (note names, status text)
