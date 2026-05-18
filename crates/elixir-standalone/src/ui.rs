//! egui window for the standalone synth (Phase 21.B6.2 — visual).
//!
//! No sliders. Rotary knobs, ADSR shape painter, filter frequency-
//! response curve, and a real piano keyboard. Mod-matrix and wave-
//! table editors land in B7+.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use eframe::egui::{
    self, vec2, Align2, Color32, FontId, Frame, Key, Pos2, Rect, Response, Rounding, Sense, Shape,
    Stroke, Ui,
};

use elixir_core::filter::FilterKind;
use elixir_core::fx::{Delay, Drive, FxSlot, Reverb};
use elixir_core::osc::{PhaseDistortionMode, SpectralMorph, UnisonStyle, MAX_UNISON};
use elixir_core::{Engine, MAX_POLYPHONY};

const ACCENT_OSC: Color32 = Color32::from_rgb(240, 200, 130);

// ─── theme tokens ────────────────────────────────────────────────────

const BG: Color32 = Color32::from_rgb(14, 16, 22);
const PANEL: Color32 = Color32::from_rgb(20, 23, 31);
const CARD: Color32 = Color32::from_rgb(26, 30, 40);
const CARD_BORDER: Color32 = Color32::from_rgb(48, 54, 70);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 236, 244);
const TEXT_DIM: Color32 = Color32::from_rgb(140, 148, 168);

const ACCENT_MASTER: Color32 = Color32::from_rgb(228, 196, 96);
const ACCENT_ENV: Color32 = Color32::from_rgb(120, 200, 230);
const ACCENT_FILTER: Color32 = Color32::from_rgb(180, 140, 240);
const ACCENT_FX: Color32 = Color32::from_rgb(255, 150, 110);
const ACCENT_KEYS: Color32 = Color32::from_rgb(140, 220, 160);

const KNOB_TRACK: Color32 = Color32::from_rgb(40, 46, 60);

// ─── entry point ─────────────────────────────────────────────────────

pub fn run(engine: Arc<Mutex<Engine>>) -> Result<(), eframe::Error> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Elixir")
        .with_inner_size([880.0, 640.0])
        .with_min_inner_size([720.0, 540.0]);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Elixir",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            Box::new(ElixirApp::new(engine))
        }),
    )
}

fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(TEXT_PRIMARY);
    v.window_fill = BG;
    v.panel_fill = PANEL;
    v.faint_bg_color = CARD;
    v.extreme_bg_color = Color32::from_rgb(10, 12, 18);

    v.widgets.noninteractive.bg_fill = CARD;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, CARD_BORDER);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.inactive.bg_fill = Color32::from_rgb(48, 54, 70);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(36, 42, 56);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.inactive.rounding = Rounding::same(6.0);
    v.widgets.hovered.bg_fill = Color32::from_rgb(70, 80, 100);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(58, 66, 84);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
    v.widgets.hovered.rounding = Rounding::same(6.0);
    v.widgets.active.bg_fill = Color32::from_rgb(100, 120, 160);
    v.widgets.active.fg_stroke = Stroke::new(2.0, TEXT_PRIMARY);
    v.widgets.active.rounding = Rounding::same(6.0);
    v.selection.bg_fill = Color32::from_rgb(80, 110, 170);
    v.window_rounding = Rounding::same(12.0);

    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = vec2(10.0, 8.0);
    style.spacing.button_padding = vec2(10.0, 5.0);
    ctx.set_style(style);
}

// ─── knob widget ─────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct KnobSpec {
    min: f32,
    max: f32,
    default: f32,
    log: bool,
    label: &'static str,
    fmt: KnobFmt,
}

#[derive(Clone, Copy)]
enum KnobFmt {
    Plain { decimals: usize },
    Hz,
    Seconds,
    Percent,
}

impl KnobSpec {
    const fn linear(min: f32, max: f32, default: f32, label: &'static str) -> Self {
        Self {
            min,
            max,
            default,
            log: false,
            label,
            fmt: KnobFmt::Plain { decimals: 2 },
        }
    }
    const fn log(min: f32, max: f32, default: f32, label: &'static str) -> Self {
        Self {
            min,
            max,
            default,
            log: true,
            label,
            fmt: KnobFmt::Plain { decimals: 2 },
        }
    }
    const fn with_fmt(mut self, f: KnobFmt) -> Self {
        self.fmt = f;
        self
    }
}

fn format_knob(v: f32, fmt: KnobFmt) -> String {
    match fmt {
        KnobFmt::Plain { decimals } => format!("{:.*}", decimals, v),
        KnobFmt::Hz => {
            if v >= 1000.0 {
                format!("{:.2}k", v / 1000.0)
            } else {
                format!("{:.0}", v)
            }
        }
        KnobFmt::Seconds => {
            if v < 0.1 {
                format!("{:.0}ms", v * 1000.0)
            } else if v < 1.0 {
                format!("{:.0}ms", v * 1000.0)
            } else {
                format!("{:.2}s", v)
            }
        }
        KnobFmt::Percent => format!("{:.0}%", v * 100.0),
    }
}

fn norm_to_value(spec: &KnobSpec, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if spec.log {
        let min_l = spec.min.max(1e-6).ln();
        let max_l = spec.max.ln();
        (min_l + (max_l - min_l) * t).exp()
    } else {
        spec.min + (spec.max - spec.min) * t
    }
}
fn value_to_norm(spec: &KnobSpec, v: f32) -> f32 {
    let v = v.clamp(spec.min, spec.max);
    if spec.log {
        let min_l = spec.min.max(1e-6).ln();
        let max_l = spec.max.ln();
        ((v.ln() - min_l) / (max_l - min_l)).clamp(0.0, 1.0)
    } else {
        ((v - spec.min) / (spec.max - spec.min)).clamp(0.0, 1.0)
    }
}

/// Rotary knob. Drag vertically to change value; double-click resets to
/// default; shift makes fine adjustments. Arc sweeps from ~-225° to +45°
/// (so straight up = full).
fn knob(ui: &mut Ui, value: &mut f32, spec: &KnobSpec, accent: Color32) -> Response {
    let size = vec2(56.0, 78.0); // knob + label + readout
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());

    if response.double_clicked() {
        *value = spec.default;
        response.mark_changed();
    }

    // Drag handling
    if response.dragged() {
        let dy = response.drag_delta().y;
        let speed = if ui.input(|i| i.modifiers.shift) {
            0.0008
        } else {
            0.004
        };
        let mut t = value_to_norm(spec, *value);
        t -= dy * speed;
        let new = norm_to_value(spec, t);
        if (new - *value).abs() > 0.0 {
            *value = new;
            response.mark_changed();
        }
    }

    let painter = ui.painter();
    let knob_center = Pos2 {
        x: rect.center().x,
        y: rect.top() + 28.0,
    };
    let radius = 22.0;

    // Background ring (track)
    painter.circle_filled(knob_center, radius + 3.0, CARD);
    painter.circle_stroke(knob_center, radius, Stroke::new(3.5, KNOB_TRACK));

    // Active arc (accent color from start to current value)
    let t = value_to_norm(spec, *value);
    let start_angle = -std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4 * 5.0; // 225°
    let end_angle = start_angle - std::f32::consts::FRAC_PI_2 * 3.0 * t; // up to 270° sweep CCW

    let arc_points: Vec<Pos2> = (0..=48)
        .map(|i| {
            let f = i as f32 / 48.0;
            let a = start_angle + (end_angle - start_angle) * f;
            Pos2 {
                x: knob_center.x + radius * a.cos(),
                y: knob_center.y - radius * a.sin(),
            }
        })
        .collect();
    if arc_points.len() >= 2 {
        painter.add(Shape::line(arc_points, Stroke::new(3.5, accent)));
    }

    // Indicator dot at the current angle
    let dot = Pos2 {
        x: knob_center.x + radius * end_angle.cos(),
        y: knob_center.y - radius * end_angle.sin(),
    };
    painter.circle_filled(dot, 3.5, accent);

    // Center cap with hover highlight
    let cap_color = if response.hovered() || response.dragged() {
        Color32::from_rgb(60, 68, 86)
    } else {
        Color32::from_rgb(40, 46, 60)
    };
    painter.circle_filled(knob_center, radius - 5.0, cap_color);

    // Label above (small) — actually below for compactness
    let value_text = format_knob(*value, spec.fmt);
    painter.text(
        Pos2 {
            x: knob_center.x,
            y: rect.top() + 56.0,
        },
        Align2::CENTER_CENTER,
        &value_text,
        FontId::monospace(11.0),
        TEXT_PRIMARY,
    );
    painter.text(
        Pos2 {
            x: knob_center.x,
            y: rect.top() + 70.0,
        },
        Align2::CENTER_CENTER,
        spec.label,
        FontId::proportional(10.0),
        TEXT_DIM,
    );

    response
}

// ─── ADSR shape painter ──────────────────────────────────────────────

fn adsr_curve(ui: &mut Ui, attack: f32, decay: f32, sustain: f32, release: f32, accent: Color32) {
    let desired = vec2(ui.available_width(), 70.0);
    let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(18, 20, 28));

    // Normalize times so the curve fits — keep proportions.
    let attack = attack.max(0.001);
    let decay = decay.max(0.001);
    let release = release.max(0.001);
    let sustain_seg = 0.7; // visual constant proportion for sustain hold
    let total_time = attack + decay + sustain_seg + release;
    let max_w = rect.width() - 16.0;
    let scale = max_w / total_time;
    let top = rect.top() + 10.0;
    let bottom = rect.bottom() - 6.0;
    let h = bottom - top;

    let start_x = rect.left() + 8.0;
    let p0 = Pos2 {
        x: start_x,
        y: bottom,
    };
    let p1 = Pos2 {
        x: start_x + attack * scale,
        y: top,
    };
    let p2 = Pos2 {
        x: p1.x + decay * scale,
        y: top + (1.0 - sustain) * h,
    };
    let p3 = Pos2 {
        x: p2.x + sustain_seg * scale,
        y: p2.y,
    };
    let p4 = Pos2 {
        x: p3.x + release * scale,
        y: bottom,
    };

    // Filled area under the curve
    let poly = vec![p0, p1, p2, p3, p4, Pos2 { x: p4.x, y: bottom }];
    let fill = Color32::from_rgba_premultiplied(accent.r() / 6, accent.g() / 6, accent.b() / 6, 80);
    painter.add(Shape::convex_polygon(poly, fill, Stroke::NONE));

    // Outline
    painter.add(Shape::line(
        vec![p0, p1, p2, p3, p4],
        Stroke::new(2.0, accent),
    ));

    // Handle dots
    for (label, p) in [("A", p1), ("D", p2), ("S", p3), ("R", p4)] {
        painter.circle_filled(p, 3.0, accent);
        painter.text(
            Pos2 {
                x: p.x,
                y: bottom + 0.0,
            },
            Align2::CENTER_BOTTOM,
            label,
            FontId::proportional(9.0),
            TEXT_DIM,
        );
    }
}

// ─── Filter response curve ───────────────────────────────────────────

fn filter_curve(ui: &mut Ui, cutoff_hz: f32, resonance: f32, accent: Color32) {
    let desired = vec2(ui.available_width(), 80.0);
    let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(18, 20, 28));

    // X axis = log frequency 20Hz..20kHz; Y = magnitude in dB roughly
    let log_min = 20f32.ln();
    let log_max = 20_000f32.ln();
    let top = rect.top() + 6.0;
    let bottom = rect.bottom() - 10.0;
    let h = bottom - top;
    let left = rect.left() + 8.0;
    let right = rect.right() - 8.0;
    let w = right - left;

    // Gridlines at decades
    for &f in &[100.0f32, 1_000.0, 10_000.0] {
        let t = (f.ln() - log_min) / (log_max - log_min);
        let x = left + w * t;
        painter.line_segment(
            [Pos2 { x, y: top }, Pos2 { x, y: bottom }],
            Stroke::new(1.0, Color32::from_rgb(28, 32, 44)),
        );
        painter.text(
            Pos2 { x, y: bottom + 4.0 },
            Align2::CENTER_TOP,
            if f >= 1000.0 {
                format!("{}k", (f / 1000.0) as i32)
            } else {
                format!("{}", f as i32)
            },
            FontId::proportional(8.0),
            TEXT_DIM,
        );
    }

    // Build a magnitude curve approximating a 2-pole LP with resonance
    // peak at the cutoff. Pure cosmetic — not the actual SVF transfer.
    let cutoff_clamped = cutoff_hz.clamp(20.0, 20_000.0);
    let peak_db = resonance * 18.0; // up to ~18 dB lift at high Q
    let mut points = Vec::with_capacity(80);
    for i in 0..=80 {
        let t = i as f32 / 80.0;
        let f = (log_min + t * (log_max - log_min)).exp();
        let ratio = f / cutoff_clamped;
        // 12 dB/oct roll-off past cutoff, plus a Q peak at the cutoff
        let mag_db_lp = -20.0 * (1.0 + ratio.powi(4)).log10() * 0.5;
        let peak = peak_db * (-((ratio.ln().powi(2)) * 6.0)).exp();
        let mag = mag_db_lp + peak;
        // Map [-30, +18] dB → [bottom, top]
        let y_norm = ((mag + 30.0) / 48.0).clamp(0.0, 1.0);
        let y = bottom - y_norm * h;
        let x = left + w * t;
        points.push(Pos2 { x, y });
    }
    // Fill under curve
    let mut poly = points.clone();
    poly.push(Pos2 {
        x: right,
        y: bottom,
    });
    poly.push(Pos2 { x: left, y: bottom });
    let fill = Color32::from_rgba_premultiplied(accent.r() / 8, accent.g() / 8, accent.b() / 8, 90);
    painter.add(Shape::convex_polygon(poly, fill, Stroke::NONE));
    painter.add(Shape::line(points, Stroke::new(2.0, accent)));

    // Cutoff marker
    let cutoff_t = ((cutoff_clamped.ln() - log_min) / (log_max - log_min)).clamp(0.0, 1.0);
    let cx = left + w * cutoff_t;
    painter.line_segment(
        [Pos2 { x: cx, y: top }, Pos2 { x: cx, y: bottom }],
        Stroke::new(1.5, accent),
    );
    painter.circle_filled(
        Pos2 {
            x: cx,
            y: top + 8.0,
        },
        4.0,
        accent,
    );
}

// ─── Piano keyboard ──────────────────────────────────────────────────

fn piano_keyboard(ui: &mut Ui, held_notes: &HashSet<u8>, accent: Color32) -> Vec<KeyEvent> {
    // Two-octave keyboard starting at C3 (note 48).
    const FIRST_NOTE: u8 = 48;
    const WHITE_COUNT: usize = 14; // 2 octaves of white keys
    const BLACK_PATTERN: &[(usize, u8)] = &[
        (0, 1),
        (1, 3),
        (3, 6),
        (4, 8),
        (5, 10),
        (7, 13),
        (8, 15),
        (10, 18),
        (11, 20),
        (12, 22),
    ];

    let desired = vec2(ui.available_width(), 80.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(18, 20, 28));

    let white_w = rect.width() / WHITE_COUNT as f32;
    let white_h = rect.height();
    let black_w = white_w * 0.6;
    let black_h = white_h * 0.62;

    let mut events = Vec::new();

    // Helper: which note is at a point?
    let note_at = |p: Pos2| -> Option<u8> {
        if !rect.contains(p) {
            return None;
        }
        // Test black keys first (they overlap whites)
        let dx = p.x - rect.left();
        for &(white_idx, semitone_offset) in BLACK_PATTERN {
            let key_center = rect.left() + (white_idx as f32 + 1.0) * white_w;
            let key_left = key_center - black_w * 0.5;
            let key_right = key_center + black_w * 0.5;
            if dx + rect.left() >= key_left
                && dx + rect.left() <= key_right
                && p.y <= rect.top() + black_h
            {
                return Some(FIRST_NOTE + semitone_offset);
            }
        }
        // White keys: lookup by index
        let widx = ((p.x - rect.left()) / white_w).floor() as i32;
        if widx < 0 || widx as usize >= WHITE_COUNT {
            return None;
        }
        let white_to_semitone = [0u8, 2, 4, 5, 7, 9, 11, 12, 14, 16, 17, 19, 21, 23];
        Some(FIRST_NOTE + white_to_semitone[widx as usize])
    };

    // Detect click / drag activations
    if response.clicked() || response.drag_started() {
        if let Some(pos) = response.interact_pointer_pos() {
            if let Some(note) = note_at(pos) {
                events.push(KeyEvent::On(note));
            }
        }
    }
    if response.drag_stopped() {
        events.push(KeyEvent::AllOff);
    }

    // Paint white keys
    let white_to_semitone = [0u8, 2, 4, 5, 7, 9, 11, 12, 14, 16, 17, 19, 21, 23];
    for i in 0..WHITE_COUNT {
        let x = rect.left() + i as f32 * white_w;
        let key_rect = Rect::from_min_size(Pos2 { x, y: rect.top() }, vec2(white_w, white_h));
        let note = FIRST_NOTE + white_to_semitone[i];
        let held = held_notes.contains(&note);
        let fill = if held {
            // tinted with accent
            Color32::from_rgb(
                ((accent.r() as u16 + 240) / 2) as u8,
                ((accent.g() as u16 + 240) / 2) as u8,
                ((accent.b() as u16 + 240) / 2) as u8,
            )
        } else {
            Color32::from_rgb(232, 232, 232)
        };
        painter.rect(
            key_rect.shrink(1.0),
            Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 4.0,
                se: 4.0,
            },
            fill,
            Stroke::new(1.0, Color32::from_rgb(40, 40, 50)),
        );
        // C label
        let semi = note % 12;
        if semi == 0 {
            painter.text(
                Pos2 {
                    x: x + white_w * 0.5,
                    y: rect.bottom() - 10.0,
                },
                Align2::CENTER_CENTER,
                format!("C{}", note as i32 / 12 - 1),
                FontId::proportional(9.0),
                Color32::from_rgb(120, 120, 130),
            );
        }
    }

    // Paint black keys on top
    for &(white_idx, semitone_offset) in BLACK_PATTERN {
        let key_center = rect.left() + (white_idx as f32 + 1.0) * white_w;
        let key_rect = Rect::from_min_size(
            Pos2 {
                x: key_center - black_w * 0.5,
                y: rect.top(),
            },
            vec2(black_w, black_h),
        );
        let note = FIRST_NOTE + semitone_offset;
        let held = held_notes.contains(&note);
        let fill = if held {
            accent
        } else {
            Color32::from_rgb(28, 30, 36)
        };
        painter.rect(
            key_rect,
            Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 3.0,
                se: 3.0,
            },
            fill,
            Stroke::new(1.0, Color32::BLACK),
        );
    }

    events
}

#[derive(Clone, Copy, Debug)]
enum KeyEvent {
    On(u8),
    AllOff,
}

// ─── app + snapshot ──────────────────────────────────────────────────

struct ElixirApp {
    engine: Arc<Mutex<Engine>>,
    keys_held: HashSet<Key>,
    mouse_note: Option<u8>,
    keyboard_octave: i32,
    snapshot: EngineSnapshot,
}

#[derive(Clone, Default)]
struct EngineSnapshot {
    master_gain: f32,
    amp_attack_secs: f32,
    amp_decay_secs: f32,
    amp_sustain: f32,
    amp_release_secs: f32,
    filter_cutoff_hz: f32,
    filter_resonance: f32,
    filter_kind: FilterKind,
    filter_drive: f32,
    filter_gain: f32,
    filter_morph_x: f32,
    filter_morph_y: f32,
    spectral_morph: SpectralMorph,
    morph_amount: f32,
    phase_distortion: PhaseDistortionMode,
    phase_amount: f32,
    unison_voices: u8,
    unison_style: UnisonStyle,
    unison_detune_cents: f32,
    drive_on: bool,
    drive_amount: f32,
    drive_mix: f32,
    delay_on: bool,
    delay_secs: f32,
    delay_feedback: f32,
    delay_mix: f32,
    reverb_on: bool,
    reverb_decay: f32,
    reverb_damping: f32,
    reverb_mix: f32,
    live_voices: usize,
    sustain_pedal: bool,
    active_notes: HashSet<u8>,
}

impl ElixirApp {
    fn new(engine: Arc<Mutex<Engine>>) -> Self {
        let snapshot = EngineSnapshot::from(&engine);
        Self {
            engine,
            keys_held: HashSet::new(),
            mouse_note: None,
            keyboard_octave: 4,
            snapshot,
        }
    }

    fn snapshot(&mut self) {
        self.snapshot = EngineSnapshot::from(&self.engine);
        // Add computer-keyboard held notes for visual highlight.
        for k in &self.keys_held {
            if let Some(sem) = key_to_semitone(*k) {
                let note = (self.keyboard_octave * 12 + 12 + sem) as u8;
                self.snapshot.active_notes.insert(note);
            }
        }
        if let Some(n) = self.mouse_note {
            self.snapshot.active_notes.insert(n);
        }
    }
}

impl EngineSnapshot {
    fn from(engine: &Arc<Mutex<Engine>>) -> Self {
        let Ok(e) = engine.lock() else {
            return Self::default();
        };
        let (drive_on, drive_amount, drive_mix) = match &e.fx_chain[0] {
            FxSlot::Drive(d) => (true, d.drive, d.mix),
            _ => (false, 2.5, 0.4),
        };
        let (delay_on, delay_secs, delay_feedback, delay_mix) = match &e.fx_chain[1] {
            FxSlot::Delay(_) => (true, 0.375, 0.45, 0.30),
            _ => (false, 0.375, 0.45, 0.30),
        };
        let (reverb_on, reverb_decay, reverb_damping, reverb_mix) = match &e.fx_chain[2] {
            FxSlot::Reverb(_) => (true, 0.85, 0.4, 0.30),
            _ => (false, 0.85, 0.4, 0.30),
        };
        let osc = e.osc_params();
        let (morph_x, morph_y) = e.filter_morph();
        Self {
            master_gain: e.master_gain(),
            amp_attack_secs: e.amp_attack_secs(),
            amp_decay_secs: e.amp_decay_secs(),
            amp_sustain: e.amp_sustain(),
            amp_release_secs: e.amp_release_secs(),
            filter_cutoff_hz: e.filter_cutoff_hz(),
            filter_resonance: e.filter_resonance(),
            filter_kind: e.filter_kind(),
            filter_drive: e.filter_drive(),
            filter_gain: e.filter_gain(),
            filter_morph_x: morph_x,
            filter_morph_y: morph_y,
            spectral_morph: osc.spectral_morph,
            morph_amount: osc.morph_amount,
            phase_distortion: osc.phase_distortion,
            phase_amount: osc.phase_amount,
            unison_voices: osc.unison_voices,
            unison_style: osc.unison_style,
            unison_detune_cents: osc.unison_detune_cents,
            drive_on,
            drive_amount,
            drive_mix,
            delay_on,
            delay_secs,
            delay_feedback,
            delay_mix,
            reverb_on,
            reverb_decay,
            reverb_damping,
            reverb_mix,
            live_voices: e.live_voice_count(),
            sustain_pedal: e.sustain_pedal(),
            active_notes: HashSet::new(),
        }
    }
}

// ─── card frame helper ───────────────────────────────────────────────

fn card<R>(ui: &mut Ui, accent: Color32, body: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::none()
        .fill(CARD)
        .stroke(Stroke::new(1.0, CARD_BORDER))
        .rounding(Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            let stripe_rect = Rect::from_min_size(
                ui.min_rect().min - vec2(14.0, 0.0),
                vec2(3.0, ui.available_height().max(40.0)),
            );
            ui.painter()
                .rect_filled(stripe_rect, Rounding::same(2.0), accent);
            body(ui)
        })
        .inner
}

fn section_title(ui: &mut Ui, label: &str, accent: Color32) {
    ui.label(
        egui::RichText::new(label)
            .color(accent)
            .strong()
            .extra_letter_spacing(2.0)
            .size(11.0),
    );
    ui.add_space(4.0);
}

// ─── update ──────────────────────────────────────────────────────────

impl eframe::App for ElixirApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_keyboard(ctx);
        self.snapshot();

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("ELIXIR")
                            .color(TEXT_PRIMARY)
                            .strong()
                            .size(22.0)
                            .extra_letter_spacing(6.0),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("v0.0.1 · phase 21")
                            .color(TEXT_DIM)
                            .small(),
                    );
                });
                ui.add_space(8.0);

                // Master + envelope
                ui.horizontal(|ui| {
                    // master card
                    Frame::none()
                        .fill(CARD)
                        .stroke(Stroke::new(1.0, CARD_BORDER))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
                        .show(ui, |ui| {
                            ui.set_min_width(140.0);
                            ui.vertical(|ui| {
                                section_title(ui, "MASTER", ACCENT_MASTER);
                                let mut v = self.snapshot.master_gain;
                                if knob(
                                    ui,
                                    &mut v,
                                    &KnobSpec::linear(0.0, 1.0, 0.30, "gain")
                                        .with_fmt(KnobFmt::Percent),
                                    ACCENT_MASTER,
                                )
                                .changed()
                                {
                                    if let Ok(mut e) = self.engine.lock() {
                                        e.set_master_gain(v);
                                    }
                                }
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{:>2}/{} voices",
                                        self.snapshot.live_voices, MAX_POLYPHONY
                                    ))
                                    .monospace()
                                    .small()
                                    .color(TEXT_DIM),
                                );
                                if self.snapshot.sustain_pedal {
                                    ui.colored_label(ACCENT_FILTER, "● sustain");
                                }
                            });
                        });

                    ui.add_space(10.0);

                    // envelope card
                    Frame::none()
                        .fill(CARD)
                        .stroke(Stroke::new(1.0, CARD_BORDER))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                section_title(ui, "AMP ENVELOPE", ACCENT_ENV);
                                adsr_curve(
                                    ui,
                                    self.snapshot.amp_attack_secs,
                                    self.snapshot.amp_decay_secs,
                                    self.snapshot.amp_sustain,
                                    self.snapshot.amp_release_secs,
                                    ACCENT_ENV,
                                );
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    let mut a = self.snapshot.amp_attack_secs;
                                    if knob(
                                        ui,
                                        &mut a,
                                        &KnobSpec::log(0.001, 4.0, 0.005, "attack")
                                            .with_fmt(KnobFmt::Seconds),
                                        ACCENT_ENV,
                                    )
                                    .changed()
                                    {
                                        if let Ok(mut e) = self.engine.lock() {
                                            e.set_amp_attack_secs(a);
                                        }
                                    }
                                    let mut d = self.snapshot.amp_decay_secs;
                                    if knob(
                                        ui,
                                        &mut d,
                                        &KnobSpec::log(0.001, 4.0, 0.12, "decay")
                                            .with_fmt(KnobFmt::Seconds),
                                        ACCENT_ENV,
                                    )
                                    .changed()
                                    {
                                        if let Ok(mut e) = self.engine.lock() {
                                            e.set_amp_decay_secs(d);
                                        }
                                    }
                                    let mut s = self.snapshot.amp_sustain;
                                    if knob(
                                        ui,
                                        &mut s,
                                        &KnobSpec::linear(0.0, 1.0, 0.7, "sustain")
                                            .with_fmt(KnobFmt::Percent),
                                        ACCENT_ENV,
                                    )
                                    .changed()
                                    {
                                        if let Ok(mut e) = self.engine.lock() {
                                            e.set_amp_sustain(s);
                                        }
                                    }
                                    let mut r = self.snapshot.amp_release_secs;
                                    if knob(
                                        ui,
                                        &mut r,
                                        &KnobSpec::log(0.001, 8.0, 0.25, "release")
                                            .with_fmt(KnobFmt::Seconds),
                                        ACCENT_ENV,
                                    )
                                    .changed()
                                    {
                                        if let Ok(mut e) = self.engine.lock() {
                                            e.set_amp_release_secs(r);
                                        }
                                    }
                                });
                            });
                        });
                });

                ui.add_space(10.0);

                // OSC card (A6)
                card(ui, ACCENT_OSC, |ui| {
                    self.draw_osc(ui);
                });

                ui.add_space(10.0);

                // Filter card with frequency response curve
                card(ui, ACCENT_FILTER, |ui| {
                    section_title(ui, "FILTER", ACCENT_FILTER);
                    filter_curve(
                        ui,
                        self.snapshot.filter_cutoff_hz,
                        self.snapshot.filter_resonance,
                        ACCENT_FILTER,
                    );
                    ui.add_space(4.0);
                    self.draw_filter_controls(ui);
                });

                ui.add_space(10.0);

                // FX chain card
                card(ui, ACCENT_FX, |ui| {
                    section_title(ui, "FX CHAIN", ACCENT_FX);
                    ui.horizontal(|ui| {
                        self.draw_drive(ui);
                        ui.separator();
                        self.draw_delay(ui);
                        ui.separator();
                        self.draw_reverb(ui);
                    });
                });

                ui.add_space(10.0);

                // Piano keyboard card
                Frame::none()
                    .fill(CARD)
                    .stroke(Stroke::new(1.0, CARD_BORDER))
                    .rounding(Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 10.0))
                    .show(ui, |ui| {
                        section_title(ui, "KEYBOARD (C3–D5)", ACCENT_KEYS);
                        // release any mouse-held note when no longer being held
                        let mouse_released = ctx.input(|i| !i.pointer.primary_down());
                        if mouse_released {
                            if let Some(n) = self.mouse_note.take() {
                                if let Ok(mut e) = self.engine.lock() {
                                    e.note_off(n);
                                }
                            }
                        }
                        let events = piano_keyboard(ui, &self.snapshot.active_notes, ACCENT_KEYS);
                        for ev in events {
                            match ev {
                                KeyEvent::On(n) => {
                                    if let Some(prev) = self.mouse_note.replace(n) {
                                        if prev != n {
                                            if let Ok(mut e) = self.engine.lock() {
                                                e.note_off(prev);
                                            }
                                        }
                                    }
                                    if let Ok(mut e) = self.engine.lock() {
                                        e.note_on(n, 100);
                                    }
                                }
                                KeyEvent::AllOff => {
                                    if let Some(n) = self.mouse_note.take() {
                                        if let Ok(mut e) = self.engine.lock() {
                                            e.note_off(n);
                                        }
                                    }
                                }
                            }
                        }
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "computer kbd octave: {}",
                                    self.keyboard_octave
                                ))
                                .small()
                                .color(TEXT_DIM),
                            );
                            if ui.small_button("◀ z").clicked() {
                                self.keyboard_octave = (self.keyboard_octave - 1).max(0);
                            }
                            if ui.small_button("x ▶").clicked() {
                                self.keyboard_octave = (self.keyboard_octave + 1).min(9);
                            }
                            if ui.small_button("All notes off").clicked() {
                                if let Ok(mut e) = self.engine.lock() {
                                    e.all_notes_off();
                                }
                                self.keys_held.clear();
                            }
                        });
                    });
            });

        ctx.request_repaint_after(std::time::Duration::from_millis(33));
    }
}

// ─── FX rows ─────────────────────────────────────────────────────────

impl ElixirApp {
    fn draw_osc(&mut self, ui: &mut Ui) {
        section_title(ui, "OSCILLATOR (A6)", ACCENT_OSC);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("morph").color(TEXT_DIM).small());
            let mut morph = self.snapshot.spectral_morph;
            egui::ComboBox::from_id_source("spectral_morph")
                .selected_text(format!("{morph:?}"))
                .show_ui(ui, |ui| {
                    for m in SpectralMorph::ALL {
                        if ui
                            .selectable_value(&mut morph, m, format!("{m:?}"))
                            .changed()
                        {
                            if let Ok(mut e) = self.engine.lock() {
                                e.set_spectral_morph(m);
                            }
                        }
                    }
                });
            let mut amt = self.snapshot.morph_amount;
            if knob(
                ui,
                &mut amt,
                &KnobSpec::linear(0.0, 1.0, 0.0, "amount").with_fmt(KnobFmt::Percent),
                ACCENT_OSC,
            )
            .changed()
            {
                if let Ok(mut e) = self.engine.lock() {
                    e.set_morph_amount(amt);
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("phase").color(TEXT_DIM).small());
            let mut mode = self.snapshot.phase_distortion;
            egui::ComboBox::from_id_source("phase_distortion")
                .selected_text(format!("{mode:?}"))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut mode, PhaseDistortionMode::Off, "Off")
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            e.set_phase_distortion(PhaseDistortionMode::Off);
                        }
                    }
                    for m in PhaseDistortionMode::ALL_A6 {
                        if ui
                            .selectable_value(&mut mode, m, format!("{m:?}"))
                            .changed()
                        {
                            if let Ok(mut e) = self.engine.lock() {
                                e.set_phase_distortion(m);
                            }
                        }
                    }
                });
            let mut amt = self.snapshot.phase_amount;
            if knob(
                ui,
                &mut amt,
                &KnobSpec::linear(0.0, 1.0, 0.0, "amount").with_fmt(KnobFmt::Percent),
                ACCENT_OSC,
            )
            .changed()
            {
                if let Ok(mut e) = self.engine.lock() {
                    e.set_phase_amount(amt);
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("unison").color(TEXT_DIM).small());
            let mut style = self.snapshot.unison_style;
            egui::ComboBox::from_id_source("unison_style")
                .selected_text(format!("{style:?}"))
                .show_ui(ui, |ui| {
                    for s in UnisonStyle::ALL {
                        if ui
                            .selectable_value(&mut style, s, format!("{s:?}"))
                            .changed()
                        {
                            if let Ok(mut e) = self.engine.lock() {
                                e.set_unison_style(s);
                            }
                        }
                    }
                });
            let mut voices = self.snapshot.unison_voices as f32;
            if knob(
                ui,
                &mut voices,
                &KnobSpec::linear(1.0, MAX_UNISON as f32, 1.0, "voices"),
                ACCENT_OSC,
            )
            .changed()
            {
                if let Ok(mut e) = self.engine.lock() {
                    e.set_unison_voices(voices.round() as u8);
                }
            }
            let mut det = self.snapshot.unison_detune_cents;
            if knob(
                ui,
                &mut det,
                &KnobSpec::linear(0.0, 100.0, 8.0, "detune"),
                ACCENT_OSC,
            )
            .changed()
            {
                if let Ok(mut e) = self.engine.lock() {
                    e.set_unison_detune_cents(det);
                }
            }
        });
    }

    fn draw_filter_controls(&mut self, ui: &mut Ui) {
        filter_curve(
            ui,
            self.snapshot.filter_cutoff_hz,
            self.snapshot.filter_resonance,
            ACCENT_FILTER,
        );
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("model").color(TEXT_DIM).small());
            let mut kind = self.snapshot.filter_kind;
            egui::ComboBox::from_id_source("filter_kind")
                .selected_text(format!("{kind:?}"))
                .show_ui(ui, |ui| {
                    for k in FilterKind::ALL {
                        if ui
                            .selectable_value(&mut kind, k, format!("{k:?}"))
                            .changed()
                        {
                            if let Ok(mut e) = self.engine.lock() {
                                e.set_filter_kind(k);
                            }
                        }
                    }
                });
        });
        ui.horizontal(|ui| {
            let mut c = self.snapshot.filter_cutoff_hz;
            if knob(
                ui,
                &mut c,
                &KnobSpec::log(50.0, 20_000.0, 8_000.0, "cutoff").with_fmt(KnobFmt::Hz),
                ACCENT_FILTER,
            )
            .changed()
            {
                if let Ok(mut e) = self.engine.lock() {
                    e.set_filter_cutoff_hz(c);
                }
            }
            let mut q = self.snapshot.filter_resonance;
            if knob(
                ui,
                &mut q,
                &KnobSpec::linear(0.0, 0.99, 0.0, "Q").with_fmt(KnobFmt::Percent),
                ACCENT_FILTER,
            )
            .changed()
            {
                if let Ok(mut e) = self.engine.lock() {
                    e.set_filter_resonance(q);
                }
            }
            let mut drive = self.snapshot.filter_drive;
            if knob(
                ui,
                &mut drive,
                &KnobSpec::linear(0.1, 16.0, 1.0, "drive"),
                ACCENT_FILTER,
            )
            .changed()
            {
                if let Ok(mut e) = self.engine.lock() {
                    e.set_filter_drive(drive);
                }
            }
            let mut gain = self.snapshot.filter_gain;
            if knob(
                ui,
                &mut gain,
                &KnobSpec::linear(0.0, 4.0, 1.0, "gain"),
                ACCENT_FILTER,
            )
            .changed()
            {
                if let Ok(mut e) = self.engine.lock() {
                    e.set_filter_gain(gain);
                }
            }
        });
        if matches!(
            self.snapshot.filter_kind,
            FilterKind::Formant | FilterKind::Phaser
        ) {
            ui.horizontal(|ui| {
                let mut mx = self.snapshot.filter_morph_x;
                if knob(
                    ui,
                    &mut mx,
                    &KnobSpec::linear(0.0, 1.0, 0.0, "morph X").with_fmt(KnobFmt::Percent),
                    ACCENT_FILTER,
                )
                .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_filter_morph(mx, self.snapshot.filter_morph_y);
                    }
                }
                let mut my = self.snapshot.filter_morph_y;
                if knob(
                    ui,
                    &mut my,
                    &KnobSpec::linear(0.0, 1.0, 0.0, "morph Y").with_fmt(KnobFmt::Percent),
                    ACCENT_FILTER,
                )
                .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_filter_morph(self.snapshot.filter_morph_x, my);
                    }
                }
            });
        }
    }

    fn draw_drive(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut on = self.snapshot.drive_on;
                if ui.checkbox(&mut on, "DRIVE").changed() {
                    if let Ok(mut e) = self.engine.lock() {
                        if on {
                            e.set_fx_slot(
                                0,
                                FxSlot::Drive(Drive::with_drive(self.snapshot.drive_amount)),
                            );
                        } else {
                            e.clear_fx_slot(0);
                        }
                    }
                }
            });
            if self.snapshot.drive_on {
                ui.horizontal(|ui| {
                    let mut amt = self.snapshot.drive_amount;
                    if knob(
                        ui,
                        &mut amt,
                        &KnobSpec::linear(0.5, 20.0, 2.5, "amount"),
                        ACCENT_FX,
                    )
                    .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Drive(d) = &mut e.fx_chain[0] {
                                d.drive = amt;
                            }
                        }
                    }
                    let mut mix = self.snapshot.drive_mix;
                    if knob(
                        ui,
                        &mut mix,
                        &KnobSpec::linear(0.0, 1.0, 0.4, "mix").with_fmt(KnobFmt::Percent),
                        ACCENT_FX,
                    )
                    .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Drive(d) = &mut e.fx_chain[0] {
                                d.mix = mix;
                            }
                        }
                    }
                });
            } else {
                ui.label(egui::RichText::new("(off)").color(TEXT_DIM).small());
            }
        });
    }

    fn draw_delay(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut on = self.snapshot.delay_on;
                if ui.checkbox(&mut on, "DELAY").changed() {
                    if let Ok(mut e) = self.engine.lock() {
                        if on {
                            let mut d = Delay::new(48_000);
                            d.set_delay_secs(self.snapshot.delay_secs, 48_000.0);
                            d.set_feedback(self.snapshot.delay_feedback);
                            d.set_mix(self.snapshot.delay_mix);
                            e.set_fx_slot(1, FxSlot::Delay(d));
                        } else {
                            e.clear_fx_slot(1);
                        }
                    }
                }
            });
            if self.snapshot.delay_on {
                ui.horizontal(|ui| {
                    let mut t = self.snapshot.delay_secs;
                    if knob(
                        ui,
                        &mut t,
                        &KnobSpec::linear(0.01, 1.0, 0.375, "time").with_fmt(KnobFmt::Seconds),
                        ACCENT_FX,
                    )
                    .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Delay(d) = &mut e.fx_chain[1] {
                                d.set_delay_secs(t, 48_000.0);
                            }
                        }
                    }
                    let mut fb = self.snapshot.delay_feedback;
                    if knob(
                        ui,
                        &mut fb,
                        &KnobSpec::linear(0.0, 0.95, 0.45, "fb").with_fmt(KnobFmt::Percent),
                        ACCENT_FX,
                    )
                    .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Delay(d) = &mut e.fx_chain[1] {
                                d.set_feedback(fb);
                            }
                        }
                    }
                    let mut mix = self.snapshot.delay_mix;
                    if knob(
                        ui,
                        &mut mix,
                        &KnobSpec::linear(0.0, 1.0, 0.3, "mix").with_fmt(KnobFmt::Percent),
                        ACCENT_FX,
                    )
                    .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Delay(d) = &mut e.fx_chain[1] {
                                d.set_mix(mix);
                            }
                        }
                    }
                });
            } else {
                ui.label(egui::RichText::new("(off)").color(TEXT_DIM).small());
            }
        });
    }

    fn draw_reverb(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut on = self.snapshot.reverb_on;
                if ui.checkbox(&mut on, "REVERB").changed() {
                    if let Ok(mut e) = self.engine.lock() {
                        if on {
                            let mut r = Reverb::new(48_000.0);
                            r.set_decay(self.snapshot.reverb_decay);
                            r.set_damping(self.snapshot.reverb_damping);
                            r.set_mix(self.snapshot.reverb_mix);
                            e.set_fx_slot(2, FxSlot::Reverb(r));
                        } else {
                            e.clear_fx_slot(2);
                        }
                    }
                }
            });
            if self.snapshot.reverb_on {
                ui.horizontal(|ui| {
                    let mut dec = self.snapshot.reverb_decay;
                    if knob(
                        ui,
                        &mut dec,
                        &KnobSpec::linear(0.0, 0.98, 0.85, "decay").with_fmt(KnobFmt::Percent),
                        ACCENT_FX,
                    )
                    .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Reverb(r) = &mut e.fx_chain[2] {
                                r.set_decay(dec);
                            }
                        }
                    }
                    let mut dmp = self.snapshot.reverb_damping;
                    if knob(
                        ui,
                        &mut dmp,
                        &KnobSpec::linear(0.0, 1.0, 0.4, "damp").with_fmt(KnobFmt::Percent),
                        ACCENT_FX,
                    )
                    .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Reverb(r) = &mut e.fx_chain[2] {
                                r.set_damping(dmp);
                            }
                        }
                    }
                    let mut mix = self.snapshot.reverb_mix;
                    if knob(
                        ui,
                        &mut mix,
                        &KnobSpec::linear(0.0, 1.0, 0.3, "mix").with_fmt(KnobFmt::Percent),
                        ACCENT_FX,
                    )
                    .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Reverb(r) = &mut e.fx_chain[2] {
                                r.set_mix(mix);
                            }
                        }
                    }
                });
            } else {
                ui.label(egui::RichText::new("(off)").color(TEXT_DIM).small());
            }
        });
    }
}

// ─── computer keyboard ───────────────────────────────────────────────

fn key_to_semitone(key: Key) -> Option<i32> {
    Some(match key {
        Key::A => 0,
        Key::W => 1,
        Key::S => 2,
        Key::E => 3,
        Key::D => 4,
        Key::F => 5,
        Key::T => 6,
        Key::G => 7,
        Key::Y => 8,
        Key::H => 9,
        Key::U => 10,
        Key::J => 11,
        Key::K => 12,
        _ => return None,
    })
}

const KEYBOARD_KEYS: &[Key] = &[
    Key::A,
    Key::W,
    Key::S,
    Key::E,
    Key::D,
    Key::F,
    Key::T,
    Key::G,
    Key::Y,
    Key::H,
    Key::U,
    Key::J,
    Key::K,
];

impl ElixirApp {
    fn process_keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(Key::Z) {
                self.keyboard_octave = (self.keyboard_octave - 1).max(0);
            }
            if i.key_pressed(Key::X) {
                self.keyboard_octave = (self.keyboard_octave + 1).min(9);
            }
            for &key in KEYBOARD_KEYS {
                let sem = key_to_semitone(key).unwrap();
                let down = i.key_down(key);
                let was_down = self.keys_held.contains(&key);
                let note = (self.keyboard_octave * 12 + 12 + sem) as u8;
                if down && !was_down {
                    self.keys_held.insert(key);
                    if let Ok(mut e) = self.engine.lock() {
                        e.note_on(note, 100);
                    }
                } else if !down && was_down {
                    self.keys_held.remove(&key);
                    if let Ok(mut e) = self.engine.lock() {
                        e.note_off(note);
                    }
                }
            }
        });
    }
}
