//! Runtime-tweakable synth parameters, lock-free.
//!
//! All parameters are atomics scaled to integer storage so the audio
//! callback can read them without locks. Ranges are documented on
//! each getter.

use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

/// Oscillator waveform. Stored as a u8 in [`SynthParams::waveform`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Waveform {
    Sine = 0,
    Saw = 1,
    Square = 2,
    Triangle = 3,
}

impl Waveform {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Waveform::Saw,
            2 => Waveform::Square,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        }
    }
}

/// MIDI-ish events pushed from the router thread into the audio thread.
#[derive(Clone, Copy, Debug)]
pub enum SynthEvent {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    AllNotesOff,
}

/// Shared parameter store. Cloneable `Arc<SynthParams>` for UI + audio.
///
/// Fixed-point encodings:
/// - Times (attack/decay/release) stored in milliseconds (AtomicU32).
/// - Linear 0..1 values (sustain level, resonance, master gain) stored
///   as parts per thousand (0..1000 → 0.0..1.0) in AtomicU32.
/// - Cutoff stored in Hz (AtomicU32).
pub struct SynthParams {
    waveform: AtomicU8,
    attack_ms: AtomicU32,
    decay_ms: AtomicU32,
    sustain_ppt: AtomicU32,
    release_ms: AtomicU32,
    cutoff_hz: AtomicU32,
    resonance_ppt: AtomicU32,
    master_gain_ppt: AtomicU32,
    enabled: std::sync::atomic::AtomicBool,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            waveform: AtomicU8::new(Waveform::Sine as u8),
            attack_ms: AtomicU32::new(5),
            decay_ms: AtomicU32::new(120),
            sustain_ppt: AtomicU32::new(700), // 0.70
            release_ms: AtomicU32::new(250),
            cutoff_hz: AtomicU32::new(6000),
            resonance_ppt: AtomicU32::new(200),   // 0.20
            master_gain_ppt: AtomicU32::new(250), // 0.25 — conservative
            enabled: std::sync::atomic::AtomicBool::new(true),
        }
    }
}

impl SynthParams {
    pub fn new() -> Self {
        Self::default()
    }

    // ─── Readers (audio-thread hot path) ─────────────────────────

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    pub fn waveform(&self) -> Waveform {
        Waveform::from_u8(self.waveform.load(Ordering::Relaxed))
    }
    pub fn attack_secs(&self) -> f32 {
        self.attack_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn decay_secs(&self) -> f32 {
        self.decay_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn sustain_level(&self) -> f32 {
        self.sustain_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn release_secs(&self) -> f32 {
        self.release_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn cutoff_hz(&self) -> f32 {
        self.cutoff_hz.load(Ordering::Relaxed) as f32
    }
    pub fn resonance(&self) -> f32 {
        self.resonance_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn master_gain(&self) -> f32 {
        self.master_gain_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }

    // ─── Writers (command thread) ────────────────────────────────

    pub fn set_enabled(&self, v: bool) {
        self.enabled.store(v, Ordering::Relaxed);
    }
    pub fn set_waveform(&self, w: Waveform) {
        self.waveform.store(w as u8, Ordering::Relaxed);
    }
    pub fn set_attack_ms(&self, ms: u32) {
        self.attack_ms.store(ms.clamp(1, 5_000), Ordering::Relaxed);
    }
    pub fn set_decay_ms(&self, ms: u32) {
        self.decay_ms.store(ms.clamp(1, 5_000), Ordering::Relaxed);
    }
    pub fn set_sustain_level(&self, v: f32) {
        let ppt = (v.clamp(0.0, 1.0) * 1000.0) as u32;
        self.sustain_ppt.store(ppt, Ordering::Relaxed);
    }
    pub fn set_release_ms(&self, ms: u32) {
        self.release_ms
            .store(ms.clamp(1, 10_000), Ordering::Relaxed);
    }
    pub fn set_cutoff_hz(&self, hz: u32) {
        self.cutoff_hz
            .store(hz.clamp(20, 20_000), Ordering::Relaxed);
    }
    pub fn set_resonance(&self, v: f32) {
        let ppt = (v.clamp(0.0, 1.0) * 1000.0) as u32;
        self.resonance_ppt.store(ppt, Ordering::Relaxed);
    }
    pub fn set_master_gain(&self, v: f32) {
        let ppt = (v.clamp(0.0, 1.0) * 1000.0) as u32;
        self.master_gain_ppt.store(ppt, Ordering::Relaxed);
    }
}
