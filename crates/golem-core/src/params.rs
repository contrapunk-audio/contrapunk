//! Runtime parameters shared between UI/control threads and the audio thread.

use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

use crate::style::Style;

/// Atomic `f32` wrapper for realtime-safe parameter sharing.
pub struct AtomicF32 {
    bits: AtomicU32,
}

impl AtomicF32 {
    pub fn new(value: f32) -> Self {
        Self {
            bits: AtomicU32::new(value.to_bits()),
        }
    }

    pub fn load(&self) -> f32 {
        f32::from_bits(self.bits.load(Ordering::Relaxed))
    }

    pub fn store(&self, value: f32) {
        self.bits.store(value.to_bits(), Ordering::Relaxed);
    }
}

/// Plain parameter snapshot consumed by [`crate::Engine`].
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct EngineParams {
    pub bpm: f32,
    pub intensity: f32,
    pub complexity: f32,
    pub swing: f32,
    pub fill_amount: f32,
    pub follow_amount: f32,
    pub master_gain: f32,
    pub style: Style,
}

impl Default for EngineParams {
    fn default() -> Self {
        Self {
            bpm: 110.0,
            intensity: 0.55,
            complexity: 0.45,
            swing: 0.08,
            fill_amount: 0.35,
            follow_amount: 0.65,
            master_gain: 0.55,
            style: Style::Rock,
        }
    }
}

/// Realtime-safe shared parameters. UI commands write these atomics;
/// the audio callback snapshots them once per block.
pub struct SharedParams {
    bpm: AtomicF32,
    intensity: AtomicF32,
    complexity: AtomicF32,
    swing: AtomicF32,
    fill_amount: AtomicF32,
    follow_amount: AtomicF32,
    master_gain: AtomicF32,
    style: AtomicU8,
}

impl Default for SharedParams {
    fn default() -> Self {
        Self::new(EngineParams::default())
    }
}

impl SharedParams {
    pub fn new(params: EngineParams) -> Self {
        Self {
            bpm: AtomicF32::new(params.bpm),
            intensity: AtomicF32::new(params.intensity),
            complexity: AtomicF32::new(params.complexity),
            swing: AtomicF32::new(params.swing),
            fill_amount: AtomicF32::new(params.fill_amount),
            follow_amount: AtomicF32::new(params.follow_amount),
            master_gain: AtomicF32::new(params.master_gain),
            style: AtomicU8::new(params.style.as_u8()),
        }
    }

    pub fn snapshot(&self) -> EngineParams {
        EngineParams {
            bpm: self.bpm.load().clamp(40.0, 240.0),
            intensity: self.intensity.load().clamp(0.0, 1.0),
            complexity: self.complexity.load().clamp(0.0, 1.0),
            swing: self.swing.load().clamp(0.0, 1.0),
            fill_amount: self.fill_amount.load().clamp(0.0, 1.0),
            follow_amount: self.follow_amount.load().clamp(0.0, 1.0),
            master_gain: self.master_gain.load().clamp(0.0, 1.2),
            style: Style::from_u8(self.style.load(Ordering::Relaxed)),
        }
    }

    pub fn set_snapshot(&self, params: EngineParams) {
        self.set_bpm(params.bpm);
        self.set_intensity(params.intensity);
        self.set_complexity(params.complexity);
        self.set_swing(params.swing);
        self.set_fill_amount(params.fill_amount);
        self.set_follow_amount(params.follow_amount);
        self.set_master_gain(params.master_gain);
        self.set_style(params.style);
    }

    pub fn set_bpm(&self, value: f32) {
        self.bpm.store(value.clamp(40.0, 240.0));
    }
    pub fn set_intensity(&self, value: f32) {
        self.intensity.store(value.clamp(0.0, 1.0));
    }
    pub fn set_complexity(&self, value: f32) {
        self.complexity.store(value.clamp(0.0, 1.0));
    }
    pub fn set_swing(&self, value: f32) {
        self.swing.store(value.clamp(0.0, 1.0));
    }
    pub fn set_fill_amount(&self, value: f32) {
        self.fill_amount.store(value.clamp(0.0, 1.0));
    }
    pub fn set_follow_amount(&self, value: f32) {
        self.follow_amount.store(value.clamp(0.0, 1.0));
    }
    pub fn set_master_gain(&self, value: f32) {
        self.master_gain.store(value.clamp(0.0, 1.2));
    }
    pub fn set_style(&self, value: Style) {
        self.style.store(value.as_u8(), Ordering::Relaxed);
    }
}
