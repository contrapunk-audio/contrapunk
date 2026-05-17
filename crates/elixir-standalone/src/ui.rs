//! egui window for the standalone synth (Phase 21.B6).
//!
//! Minimum viable UI: knobs for master, ADSR, filter, FX mixes, plus
//! a computer-keyboard input grid. Mod-matrix and wavetable editors
//! land in B7+.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use eframe::egui::{self, Color32, Key};

use elixir_core::fx::{Delay, Drive, FxSlot, Reverb};
use elixir_core::Engine;

pub fn run(engine: Arc<Mutex<Engine>>) -> Result<(), eframe::Error> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Elixir")
        .with_inner_size([520.0, 660.0])
        .with_min_inner_size([460.0, 520.0]);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Elixir",
        options,
        Box::new(|_cc| Box::new(ElixirApp::new(engine))),
    )
}

struct ElixirApp {
    engine: Arc<Mutex<Engine>>,
    keys_held: HashSet<Key>,
    keyboard_octave: i32,
    // Cached snapshots of the engine state — read once per frame so
    // sliders don't bounce against the lock.
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
}

impl ElixirApp {
    fn new(engine: Arc<Mutex<Engine>>) -> Self {
        let snapshot = EngineSnapshot::from(&engine);
        Self {
            engine,
            keys_held: HashSet::new(),
            keyboard_octave: 4,
            snapshot,
        }
    }

    fn snapshot(&mut self) {
        self.snapshot = EngineSnapshot::from(&self.engine);
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
        Self {
            master_gain: e.master_gain(),
            amp_attack_secs: e.amp_attack_secs(),
            amp_decay_secs: e.amp_decay_secs(),
            amp_sustain: e.amp_sustain(),
            amp_release_secs: e.amp_release_secs(),
            filter_cutoff_hz: e.filter_cutoff_hz(),
            filter_resonance: e.filter_resonance(),
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
        }
    }
}

impl eframe::App for ElixirApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_keyboard(ctx);
        self.snapshot();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Elixir");
            ui.label(
                egui::RichText::new("Phase 21 prototype — knobs change the engine live.")
                    .small()
                    .color(Color32::GRAY),
            );
            ui.separator();

            // Master
            ui.horizontal(|ui| {
                ui.label("Master");
                let mut v = self.snapshot.master_gain;
                if ui
                    .add(egui::Slider::new(&mut v, 0.0..=1.0).text("gain"))
                    .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_master_gain(v);
                    }
                }
                ui.label(format!("voices: {:>2}", self.snapshot.live_voices));
                if self.snapshot.sustain_pedal {
                    ui.colored_label(Color32::LIGHT_BLUE, "[sustain]");
                }
            });

            ui.separator();
            ui.label(egui::RichText::new("AMP ENVELOPE").strong());
            ui.horizontal(|ui| {
                let mut a = self.snapshot.amp_attack_secs;
                if ui
                    .add(
                        egui::Slider::new(&mut a, 0.001..=4.0)
                            .text("A")
                            .logarithmic(true),
                    )
                    .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_amp_attack_secs(a);
                    }
                }
                let mut d = self.snapshot.amp_decay_secs;
                if ui
                    .add(
                        egui::Slider::new(&mut d, 0.001..=4.0)
                            .text("D")
                            .logarithmic(true),
                    )
                    .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_amp_decay_secs(d);
                    }
                }
            });
            ui.horizontal(|ui| {
                let mut s = self.snapshot.amp_sustain;
                if ui
                    .add(egui::Slider::new(&mut s, 0.0..=1.0).text("S"))
                    .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_amp_sustain(s);
                    }
                }
                let mut r = self.snapshot.amp_release_secs;
                if ui
                    .add(
                        egui::Slider::new(&mut r, 0.001..=8.0)
                            .text("R")
                            .logarithmic(true),
                    )
                    .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_amp_release_secs(r);
                    }
                }
            });

            ui.separator();
            ui.label(egui::RichText::new("FILTER").strong());
            ui.horizontal(|ui| {
                let mut c = self.snapshot.filter_cutoff_hz;
                if ui
                    .add(
                        egui::Slider::new(&mut c, 50.0..=20_000.0)
                            .text("cutoff (Hz)")
                            .logarithmic(true),
                    )
                    .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_filter_cutoff_hz(c);
                    }
                }
                let mut q = self.snapshot.filter_resonance;
                if ui
                    .add(egui::Slider::new(&mut q, 0.0..=0.99).text("Q"))
                    .changed()
                {
                    if let Ok(mut e) = self.engine.lock() {
                        e.set_filter_resonance(q);
                    }
                }
            });

            ui.separator();
            ui.label(egui::RichText::new("FX").strong());

            // Drive slot
            ui.horizontal(|ui| {
                let mut on = self.snapshot.drive_on;
                if ui.checkbox(&mut on, "Drive").changed() {
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
                if on {
                    let mut amt = self.snapshot.drive_amount;
                    if ui
                        .add(egui::Slider::new(&mut amt, 0.5..=20.0).text("amount"))
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Drive(d) = &mut e.fx_chain[0] {
                                d.drive = amt;
                            }
                        }
                    }
                    let mut mix = self.snapshot.drive_mix;
                    if ui
                        .add(egui::Slider::new(&mut mix, 0.0..=1.0).text("mix"))
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Drive(d) = &mut e.fx_chain[0] {
                                d.mix = mix;
                            }
                        }
                    }
                }
            });

            // Delay slot
            ui.horizontal(|ui| {
                let mut on = self.snapshot.delay_on;
                if ui.checkbox(&mut on, "Delay").changed() {
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
                    if ui
                        .add(egui::Slider::new(&mut t, 0.01..=1.0).text("time (s)"))
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Delay(d) = &mut e.fx_chain[1] {
                                d.set_delay_secs(t, 48_000.0);
                            }
                        }
                    }
                    let mut fb = self.snapshot.delay_feedback;
                    if ui
                        .add(egui::Slider::new(&mut fb, 0.0..=0.95).text("fb"))
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Delay(d) = &mut e.fx_chain[1] {
                                d.set_feedback(fb);
                            }
                        }
                    }
                    let mut mix = self.snapshot.delay_mix;
                    if ui
                        .add(egui::Slider::new(&mut mix, 0.0..=1.0).text("mix"))
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Delay(d) = &mut e.fx_chain[1] {
                                d.set_mix(mix);
                            }
                        }
                    }
                });
            }

            // Reverb slot
            ui.horizontal(|ui| {
                let mut on = self.snapshot.reverb_on;
                if ui.checkbox(&mut on, "Reverb").changed() {
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
                    if ui
                        .add(egui::Slider::new(&mut dec, 0.0..=0.98).text("decay"))
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Reverb(r) = &mut e.fx_chain[2] {
                                r.set_decay(dec);
                            }
                        }
                    }
                    let mut dmp = self.snapshot.reverb_damping;
                    if ui
                        .add(egui::Slider::new(&mut dmp, 0.0..=1.0).text("damp"))
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Reverb(r) = &mut e.fx_chain[2] {
                                r.set_damping(dmp);
                            }
                        }
                    }
                    let mut mix = self.snapshot.reverb_mix;
                    if ui
                        .add(egui::Slider::new(&mut mix, 0.0..=1.0).text("mix"))
                        .changed()
                    {
                        if let Ok(mut e) = self.engine.lock() {
                            if let FxSlot::Reverb(r) = &mut e.fx_chain[2] {
                                r.set_mix(mix);
                            }
                        }
                    }
                });
            }

            ui.separator();
            ui.label(egui::RichText::new("KEYBOARD").strong());
            ui.horizontal(|ui| {
                ui.label(format!("octave: {}", self.keyboard_octave));
                ui.label("(z = down, x = up)");
                if ui.button("All notes off").clicked() {
                    if let Ok(mut e) = self.engine.lock() {
                        e.all_notes_off();
                    }
                    self.keys_held.clear();
                }
            });
            ui.label(
                egui::RichText::new(
                    "a w s e d f t g y h u j k → chromatic. Click the window first to focus.",
                )
                .small()
                .color(Color32::GRAY),
            );
        });

        // Keep redrawing so input polling stays live without user motion.
        ctx.request_repaint_after(std::time::Duration::from_millis(33));
    }
}

const KEYBOARD_KEYS: &[(Key, i32)] = &[
    (Key::A, 0),
    (Key::W, 1),
    (Key::S, 2),
    (Key::E, 3),
    (Key::D, 4),
    (Key::F, 5),
    (Key::T, 6),
    (Key::G, 7),
    (Key::Y, 8),
    (Key::H, 9),
    (Key::U, 10),
    (Key::J, 11),
    (Key::K, 12),
];

impl ElixirApp {
    fn process_keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Octave shift
            if i.key_pressed(Key::Z) {
                self.keyboard_octave = (self.keyboard_octave - 1).max(0);
            }
            if i.key_pressed(Key::X) {
                self.keyboard_octave = (self.keyboard_octave + 1).min(9);
            }

            // Note keys
            for (key, semitone) in KEYBOARD_KEYS.iter().copied() {
                let down = i.key_down(key);
                let was_down = self.keys_held.contains(&key);
                let note = (self.keyboard_octave * 12 + 12 + semitone) as u8;
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
