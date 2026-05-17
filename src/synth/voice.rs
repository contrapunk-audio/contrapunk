//! 8-voice polyphonic synth. Runs on the audio thread; no allocations,
//! no locks.

use std::f32::consts::TAU;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use super::params::{SynthEvent, SynthParams, Waveform};
use crate::chain::{AudioBlock, MidiBlockEvent};

const MAX_VOICES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnvStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy)]
struct Voice {
    active: bool,
    note: u8,
    velocity: f32,
    phase: f32,
    phase_inc: f32,
    env: f32,
    stage: EnvStage,
    lp_state: f32,
    /// Monotonic age counter for voice-stealing when pool is full.
    /// Set to the synth's note-on counter on retrigger.
    age: u64,
}

impl Voice {
    const fn idle() -> Self {
        Self {
            active: false,
            note: 0,
            velocity: 0.0,
            phase: 0.0,
            phase_inc: 0.0,
            env: 0.0,
            stage: EnvStage::Idle,
            lp_state: 0.0,
            age: 0,
        }
    }

    fn trigger(&mut self, note: u8, velocity: u8, sample_rate: f32, age: u64) {
        self.active = true;
        self.note = note;
        self.velocity = (velocity as f32 / 127.0).clamp(0.0, 1.0);
        self.phase = 0.0;
        self.phase_inc = midi_to_freq(note) * TAU / sample_rate;
        self.env = 0.0;
        self.stage = EnvStage::Attack;
        self.lp_state = 0.0;
        self.age = age;
    }

    fn release(&mut self) {
        if self.active {
            self.stage = EnvStage::Release;
        }
    }
}

/// Polyphonic synth. Lives in the audio-owner thread; render() is
/// called from the cpal callback each block.
pub struct Synth {
    voices: [Voice; MAX_VOICES],
    params: Arc<SynthParams>,
    events: Receiver<SynthEvent>,
    sample_rate: f32,
    note_counter: u64,
}

impl Synth {
    pub fn new(params: Arc<SynthParams>, events: Receiver<SynthEvent>, sample_rate: u32) -> Self {
        Self {
            voices: [Voice::idle(); MAX_VOICES],
            params,
            events,
            sample_rate: sample_rate as f32,
            note_counter: 0,
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        self.note_counter = self.note_counter.wrapping_add(1);
        let age = self.note_counter;

        // Retrigger existing voice if same note.
        for v in self.voices.iter_mut() {
            if v.active && v.note == note && v.stage != EnvStage::Release {
                v.trigger(note, velocity, self.sample_rate, age);
                return;
            }
        }

        // Pick a free voice index; fallback to the oldest (steal).
        let idx = match self.voices.iter().position(|v| !v.active) {
            Some(i) => i,
            None => self
                .voices
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| v.age)
                .map(|(i, _)| i)
                .unwrap_or(0),
        };
        self.voices[idx].trigger(note, velocity, self.sample_rate, age);
    }

    fn note_off(&mut self, note: u8) {
        for v in self.voices.iter_mut() {
            if v.active && v.note == note && v.stage != EnvStage::Release {
                v.release();
            }
        }
    }

    fn all_notes_off(&mut self) {
        for v in self.voices.iter_mut() {
            v.release();
        }
    }

    fn process_events(&mut self) {
        while let Ok(ev) = self.events.try_recv() {
            match ev {
                SynthEvent::NoteOn { note, velocity } => self.note_on(note, velocity),
                SynthEvent::NoteOff { note } => self.note_off(note),
                SynthEvent::AllNotesOff => self.all_notes_off(),
            }
        }
    }

    /// Render `output.len() / channels` frames. Buffer is interleaved:
    /// the same mono signal is written to every channel per frame.
    ///
    /// This writes INTO `output`, overwriting whatever was there. The
    /// synth is always the first block in the chain, so that's fine.
    pub fn render(&mut self, output: &mut [f32], channels: usize) {
        self.process_events();

        if channels == 0 {
            return;
        }

        if !self.params.enabled() {
            // Short-circuit: keep envelopes idle so we don't resume
            // mid-note if re-enabled during a held note.
            for v in self.voices.iter_mut() {
                if v.active {
                    v.active = false;
                    v.stage = EnvStage::Idle;
                }
            }
            for s in output.iter_mut() {
                *s = 0.0;
            }
            return;
        }

        let frames = output.len() / channels;
        let wf = self.params.waveform();
        let attack = self.params.attack_secs().max(0.0005);
        let decay = self.params.decay_secs().max(0.0005);
        let sustain = self.params.sustain_level();
        let release = self.params.release_secs().max(0.0005);
        let master = self.params.master_gain();
        let cutoff = self.params.cutoff_hz();

        // Per-sample delta for envelope linear ramps.
        let attack_delta = 1.0 / (attack * self.sample_rate);
        let decay_delta = (1.0 - sustain) / (decay * self.sample_rate);
        let release_delta_base = 1.0 / (release * self.sample_rate);

        // One-pole low-pass alpha: y += a * (x - y)
        // a = 1 - exp(-2*pi*cutoff/sr)
        let alpha = 1.0 - (-TAU * cutoff / self.sample_rate).exp();

        for frame_idx in 0..frames {
            let mut mix = 0.0;
            for v in self.voices.iter_mut() {
                if !v.active {
                    continue;
                }

                // --- Envelope ---
                match v.stage {
                    EnvStage::Attack => {
                        v.env += attack_delta;
                        if v.env >= 1.0 {
                            v.env = 1.0;
                            v.stage = EnvStage::Decay;
                        }
                    }
                    EnvStage::Decay => {
                        v.env -= decay_delta;
                        if v.env <= sustain {
                            v.env = sustain;
                            v.stage = EnvStage::Sustain;
                        }
                    }
                    EnvStage::Sustain => {
                        // Hold at sustain until note_off.
                    }
                    EnvStage::Release => {
                        v.env -= release_delta_base;
                        if v.env <= 0.0 {
                            v.env = 0.0;
                            v.stage = EnvStage::Idle;
                            v.active = false;
                            continue;
                        }
                    }
                    EnvStage::Idle => {
                        v.active = false;
                        continue;
                    }
                }

                // --- Oscillator ---
                let osc = render_osc(wf, v.phase);
                v.phase += v.phase_inc;
                if v.phase >= TAU {
                    v.phase -= TAU;
                }

                // --- Low-pass filter (one-pole) ---
                let pre = osc * v.env * v.velocity;
                v.lp_state += alpha * (pre - v.lp_state);

                mix += v.lp_state;
            }

            // Soft ceiling: simple tanh-like clipper so heavy chords don't
            // explode. Cheap and mostly transparent at the levels we'd
            // see with ≤8 voices at master 0.25.
            let mixed = (mix * master).tanh();

            let base = frame_idx * channels;
            for ch in 0..channels {
                output[base + ch] = mixed;
            }
        }
    }
}

impl AudioBlock for Synth {
    fn name(&self) -> &str {
        "Synth"
    }

    fn type_id(&self) -> &str {
        "builtin.synth"
    }

    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        self.render(buffer, channels);
    }

    fn midi_event(&mut self, event: MidiBlockEvent) {
        // Translate the chain-level event into an internal SynthEvent
        // applied immediately. This bypasses the mpsc queue used by
        // the router thread path; both are valid entry points.
        match event {
            MidiBlockEvent::NoteOn { note, velocity } => self.note_on(note, velocity),
            MidiBlockEvent::NoteOff { note } => self.note_off(note),
            MidiBlockEvent::AllNotesOff => self.all_notes_off(),
            // Legacy synth has no sustain-pedal modelling; the router
            // thread historically held the keys until the user released
            // the pedal. Ignore here; Elixir handles it.
            MidiBlockEvent::SustainPedal { .. } => {}
        }
    }

    fn reset(&mut self) {
        self.all_notes_off();
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
    }

    fn enabled(&self) -> bool {
        self.params.enabled()
    }
}

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2f32.powf((note as f32 - 69.0) / 12.0)
}

fn render_osc(wf: Waveform, phase: f32) -> f32 {
    match wf {
        Waveform::Sine => phase.sin(),
        Waveform::Saw => {
            // Naive saw ramp: phase 0..2π → -1..1
            (phase / std::f32::consts::PI) - 1.0
        }
        Waveform::Square => {
            if phase < std::f32::consts::PI {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Triangle => {
            let p = phase / std::f32::consts::PI; // 0..2
            if p < 1.0 {
                2.0 * p - 1.0 // -1 → 1
            } else {
                3.0 - 2.0 * p // 1 → -1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn mk_synth() -> (Synth, mpsc::Sender<SynthEvent>) {
        let (tx, rx) = mpsc::channel();
        let params = Arc::new(SynthParams::default());
        let synth = Synth::new(params, rx, 48_000);
        (synth, tx)
    }

    #[test]
    fn silence_when_idle() {
        let (mut s, _tx) = mk_synth();
        let mut buf = [0.0f32; 256];
        s.render(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn note_on_produces_sound() {
        let (mut s, tx) = mk_synth();
        tx.send(SynthEvent::NoteOn {
            note: 60,
            velocity: 100,
        })
        .unwrap();

        // Render long enough to clear the attack ramp.
        let mut buf = vec![0.0f32; 48_000]; // 1 sec of mono
        s.render(&mut buf, 1);

        // At least one sample should be non-zero.
        let max_abs = buf.iter().fold(0.0f32, |a, b| a.max(b.abs()));
        assert!(
            max_abs > 0.01,
            "expected audible output, got peak {}",
            max_abs
        );
    }

    #[test]
    fn note_off_releases() {
        let (mut s, tx) = mk_synth();
        tx.send(SynthEvent::NoteOn {
            note: 60,
            velocity: 100,
        })
        .unwrap();
        let mut buf = vec![0.0f32; 24_000];
        s.render(&mut buf, 1);
        // Send note-off and render enough to release.
        tx.send(SynthEvent::NoteOff { note: 60 }).unwrap();
        let mut tail = vec![0.0f32; 48_000];
        s.render(&mut tail, 1);
        // After release, voice should be idle.
        let any_active = s.voices.iter().any(|v| v.active);
        assert!(!any_active, "voice should be idle after release");
    }

    #[test]
    fn voice_steal_does_not_panic() {
        let (mut s, tx) = mk_synth();
        for n in 60..=69 {
            tx.send(SynthEvent::NoteOn {
                note: n,
                velocity: 100,
            })
            .unwrap();
        }
        let mut buf = vec![0.0f32; 1024];
        s.render(&mut buf, 1);
        // 10 note-ons, 8 voices → 2 stolen. All 8 should be active now.
        let active = s.voices.iter().filter(|v| v.active).count();
        assert_eq!(active, 8);
    }

    #[test]
    fn params_disabled_silences_output() {
        let (mut s, tx) = mk_synth();
        tx.send(SynthEvent::NoteOn {
            note: 60,
            velocity: 100,
        })
        .unwrap();
        s.params.set_enabled(false);
        let mut buf = vec![0.0f32; 1024];
        s.render(&mut buf, 1);
        assert!(buf.iter().all(|&x| x == 0.0));
    }
}
