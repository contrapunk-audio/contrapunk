# Phase 3: GUI and Distribution - Research

**Researched:** 2026-01-28
**Domain:** egui/eframe GUI framework, Rust binary distribution
**Confidence:** HIGH

## Summary

This phase integrates a native GUI using egui/eframe (already decided) and packages the application as a single binary. egui is an immediate-mode GUI library where UI is rebuilt each frame, eliminating callback complexity and state sync issues. The existing MIDI routing runs in a background thread while the GUI displays state and receives user input.

The key architectural challenge is bridging the background MIDI thread (which uses `std::sync::mpsc` channels) with the egui event loop. This is solved by storing shared state in `Arc<Mutex<T>>` and calling `ctx.request_repaint()` when data changes. The application compiles to a single binary by default with Rust's static linking.

**Primary recommendation:** Use `Arc<Mutex<AppState>>` for shared state between MIDI thread and GUI, with `ctx.request_repaint()` triggered on note events for real-time display updates.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| egui | 0.33.x | Immediate-mode GUI widgets | Official Rust immediate-mode GUI, simple API, pure Rust |
| eframe | 0.33.x | Native window framework for egui | Official egui framework, handles window, input, rendering |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| egui_extras | 0.33.x | Additional widgets (Table, etc.) | For tables, images, strips - not needed for basic UI |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| eframe | iced, tauri | egui is simpler, immediate-mode; user decision locked to egui |
| egui_extras | Custom widgets | egui_extras not needed for ComboBox/basic widgets |

**Installation (add to Cargo.toml):**
```toml
[dependencies]
eframe = "0.33"
# egui is re-exported from eframe, no need to add separately
```

**Minimum Rust version:** 1.88.0 or later required for egui 0.33.x

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs          # CLI or GUI entry based on feature/args
├── app.rs           # NEW: eframe App impl, owns AppState
├── router.rs        # MODIFY: Accept Arc<Mutex<AppState>> for shared state
├── harmony/         # UNCHANGED: Pure harmony logic
└── midi/            # UNCHANGED: MIDI I/O
```

### Pattern 1: Shared State with Arc<Mutex<T>>
**What:** GUI and MIDI thread share state via `Arc<Mutex<AppState>>`
**When to use:** Always for this app - MIDI events need to update GUI display
**Example:**
```rust
// Source: https://github.com/emilk/egui/discussions/1428
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub key: Key,
    pub mode: HarmonyMode,
    pub active_notes: Vec<Note>,  // Currently held notes
    pub input_port: Option<usize>,
    pub output_ports: Vec<usize>,
    // Device lists for selection
    pub available_inputs: Vec<(usize, String)>,
    pub available_outputs: Vec<(usize, String)>,
}

pub struct ContrapunkApp {
    state: Arc<Mutex<AppState>>,
    ctx: Option<egui::Context>,  // For request_repaint from MIDI thread
}
```

### Pattern 2: Request Repaint from Background Thread
**What:** MIDI callback triggers GUI repaint when notes change
**When to use:** Whenever MIDI events should update display
**Example:**
```rust
// Source: https://docs.rs/egui/latest/egui/struct.Context.html
// In MIDI callback or processing thread:
fn on_midi_event(state: &Arc<Mutex<AppState>>, ctx: &egui::Context, note: Note) {
    {
        let mut s = state.lock().unwrap();
        s.active_notes.push(note);
    }
    ctx.request_repaint();  // Wake up GUI thread
}
```

### Pattern 3: eframe App Structure
**What:** Standard eframe::App implementation
**When to use:** Always - this is the entry point
**Example:**
```rust
// Source: https://docs.rs/eframe/latest/eframe/
impl eframe::App for ContrapunkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Store context for background thread repaint requests
        if self.ctx.is_none() {
            self.ctx = Some(ctx.clone());
        }

        // Lock state briefly for reading
        let state = self.state.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Contrapunk");
            ui.label(format!("Key: {}", state.key));
            ui.label(format!("Mode: {}", state.mode.description()));
            // ... widgets
        });
    }
}
```

### Pattern 4: Panel Layout Order
**What:** Panels must be added in specific order: Top/Bottom, Left/Right, then Central
**When to use:** Always when using multiple panels
**Example:**
```rust
// Source: https://docs.rs/egui/latest/egui/containers/panel/index.html
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Top panel FIRST
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.heading("Contrapunk");
    });

    // Side panel SECOND (if needed)
    egui::SidePanel::left("config").show(ctx, |ui| {
        // Configuration controls
    });

    // Central panel ALWAYS LAST
    egui::CentralPanel::default().show(ctx, |ui| {
        // Main content
    });
}
```

### Anti-Patterns to Avoid
- **Blocking in update():** Never call `.await` or long-running operations in `update()`. Use channels or spawn threads.
- **Forgetting request_repaint():** Without `ctx.request_repaint()`, background changes won't show until user interaction.
- **Holding Mutex during UI render:** Lock briefly, clone needed data, release lock, then render. Avoid holding lock across widget calls.
- **Wrong panel order:** Adding CentralPanel before SidePanel causes layout issues.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dropdown selection | Custom popup logic | `egui::ComboBox` | Handles keyboard, scroll, focus correctly |
| Device list refresh | Manual polling | Callback on button click + `request_repaint()` | Simpler, user-controlled |
| Window/app lifecycle | Manual window creation | `eframe::run_native()` | Handles all platforms, input, rendering |
| State persistence | Manual file I/O | eframe's `persistence` feature (optional) | Built-in, cross-platform storage |

**Key insight:** egui provides ComboBox, SelectableLabel, and all needed widgets. The complexity is in threading/state, not widget implementation.

## Common Pitfalls

### Pitfall 1: GUI Doesn't Update When MIDI Events Arrive
**What goes wrong:** Notes play but display shows stale state
**Why it happens:** egui only repaints on user interaction by default (power saving)
**How to avoid:** Call `ctx.request_repaint()` from MIDI callback when state changes
**Warning signs:** Display works when moving mouse, frozen otherwise

### Pitfall 2: Deadlock on Mutex
**What goes wrong:** App hangs when MIDI event arrives during UI render
**Why it happens:** UI holds lock while MIDI callback tries to acquire it
**How to avoid:** Lock briefly, clone data, release lock before rendering:
```rust
let (key, mode, notes) = {
    let s = self.state.lock().unwrap();
    (s.key, s.mode, s.active_notes.clone())
};
// Now render with local copies, no lock held
```
**Warning signs:** App freezes randomly during MIDI input

### Pitfall 3: MIDI Thread Panics on Context
**What goes wrong:** `request_repaint()` called before GUI initializes
**Why it happens:** MIDI thread starts before first `update()` call
**How to avoid:** Use `Option<egui::Context>` and set it in first `update()` call:
```rust
if let Some(ctx) = &self.ctx {
    ctx.request_repaint();
}
```
**Warning signs:** Panic on startup with MIDI device connected

### Pitfall 4: Immediate Mode State Confusion
**What goes wrong:** Widget state (e.g., ComboBox selection) resets each frame
**Why it happens:** Not storing state in app struct, expecting GUI to remember
**How to avoid:** All persistent state must be in your `AppState` struct. egui just displays it.
**Warning signs:** Dropdown resets when clicking elsewhere

### Pitfall 5: Linux Build Failures
**What goes wrong:** Compilation fails on Linux CI/CD
**Why it happens:** Missing system dependencies for wgpu/gtk
**How to avoid:** Document required packages:
```bash
# Ubuntu/Debian
sudo apt-get install -y libclang-dev libgtk-3-dev libxcb-render0-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```
**Warning signs:** Linker errors mentioning gtk, xcb, or xkbcommon

## Code Examples

Verified patterns from official sources:

### Basic eframe Application Setup
```rust
// Source: https://docs.rs/eframe/latest/eframe/
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 600.0])
            .with_min_inner_size([300.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Contrapunk",
        native_options,
        Box::new(|cc| Ok(Box::new(ContrapunkApp::new(cc)))),
    )
}
```

### ComboBox for Selection
```rust
// Source: https://docs.rs/egui/latest/egui/containers/struct.ComboBox.html
let keys = Key::all();
egui::ComboBox::from_label("Key")
    .selected_text(format!("{}", state.key))
    .show_ui(ui, |ui| {
        for key in &keys {
            ui.selectable_value(&mut state.key, *key, format!("{}", key));
        }
    });
```

### Device Selection with Index
```rust
// Source: https://docs.rs/egui/latest/egui/containers/struct.ComboBox.html
let devices = &state.available_inputs;
let mut selected = state.input_port.unwrap_or(0);

egui::ComboBox::from_label("MIDI Input")
    .selected_text(
        devices.get(selected)
            .map(|(_, name)| name.as_str())
            .unwrap_or("None")
    )
    .show_index(ui, &mut selected, devices.len(), |i| {
        devices[i].1.clone()
    });

state.input_port = Some(selected);
```

### Displaying Active Notes
```rust
// Source: https://docs.rs/egui/latest/egui/struct.Ui.html
ui.group(|ui| {
    ui.label("Active Notes:");
    ui.horizontal_wrapped(|ui| {
        for note in &state.active_notes {
            ui.label(format!("{:?}", note));
        }
        if state.active_notes.is_empty() {
            ui.label("(none)");
        }
    });
});
```

### Window Title and Icon Setup
```rust
// Source: https://docs.rs/eframe/latest/eframe/struct.NativeOptions.html
let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([400.0, 600.0])
        .with_title("Contrapunk - MIDI Harmony Generator"),
        // Note: Window icons unsupported on macOS
    ..Default::default()
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `epi::Frame::request_repaint()` | `egui::Context::request_repaint()` | egui 0.18 | Context is now the source of truth |
| `from_id_source()` | `from_id_salt()` | egui 0.28+ | Naming clarification |
| Separate egui + eframe versions | eframe re-exports egui | Always | Only add eframe to Cargo.toml |

**Deprecated/outdated:**
- `epi` crate: Merged into eframe
- `Frame::request_repaint()`: Use `Context::request_repaint()` instead
- `from_id_source()`: Use `from_id_salt()` for ComboBox identifiers

## Release Build Configuration

For single binary distribution with minimal size:

```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # Optimize for size (or "3" for speed)
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization, slower compile
strip = true         # Remove symbols
panic = "abort"      # Smaller binary, no unwinding
```

**Platform notes:**
- macOS: Works out of box. Set `MACOSX_DEPLOYMENT_TARGET=10.7` for older macOS support
- Windows: Works out of box
- Linux: Requires system libraries at runtime (gtk3, xcb). For fully static binary, would need musl target (not recommended for GUI apps)

## Open Questions

Things that couldn't be fully resolved:

1. **Hot-reload device lists**
   - What we know: Can refresh on button click
   - What's unclear: Whether to auto-detect device connect/disconnect
   - Recommendation: Start with manual refresh button, can add auto-detect later

2. **Start/Stop routing from GUI**
   - What we know: Current router runs blocking loop
   - What's unclear: Best pattern for start/stop/restart
   - Recommendation: Refactor router to be spawnable/stoppable via channel commands

3. **Error display in GUI**
   - What we know: Errors currently go to stderr
   - What's unclear: Best UX for showing MIDI errors
   - Recommendation: Add `last_error: Option<String>` to AppState, display in UI

## Sources

### Primary (HIGH confidence)
- [eframe 0.33 documentation](https://docs.rs/eframe/latest/eframe/) - NativeOptions, App trait, run_native
- [egui 0.33 documentation](https://docs.rs/egui/latest/egui/) - Widgets, Context, ComboBox, panels
- [egui GitHub](https://github.com/emilk/egui) - README, architecture overview

### Secondary (MEDIUM confidence)
- [egui discussions on threading](https://github.com/emilk/egui/discussions/1428) - Arc<Mutex> patterns
- [egui discussions on repaint](https://github.com/emilk/egui/discussions/995) - Background thread updates
- [LogRocket egui tutorial](https://blog.logrocket.com/building-cross-platform-gui-apps-rust-using-egui/) - Full app architecture
- [MIDI GUI in Rust blog](https://ntietz.com/blog/beginning-rust-midi-gui/) - MIDI+egui integration patterns

### Tertiary (LOW confidence)
- WebSearch results for "egui common pitfalls" - Community experience (needs validation)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official docs verified eframe 0.33.x, egui 0.33.x
- Architecture: HIGH - Multiple sources confirm Arc<Mutex> + request_repaint pattern
- Pitfalls: MEDIUM - Based on community discussions and docs, not firsthand testing

**Research date:** 2026-01-28
**Valid until:** 2026-02-28 (egui has frequent releases, check for 0.34.x)
