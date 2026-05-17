//! Elixir DSP engine — core crate.
//!
//! See `ELIXIR-DESIGN.md` and `ELIXIR-PLAN.md` at the repo root for the
//! full architecture and the phased rollout. The engine grows
//! incrementally through phases A0..A6 without breaking the
//! [`Engine::process`] signature locked at A0.
//!
//! Current phase: **A1** — single-voice sine oscillator, ADSR envelope,
//! MIDI note routing.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

pub mod env;
pub mod osc;
pub mod tables;
pub mod util;
pub mod voice;

use crate::tables::SineTable;
use crate::voice::Voice;

/// The top-level Elixir engine.
///
/// One instance per audio thread. Owns every per-voice and global
/// processing block. A1 has one voice; A2 grows the field to a pool of
/// `[Voice; 16]` SIMD-packed via `AggregateVoice`.
pub struct Engine {
    sample_rate: u32,
    max_block: usize,
    voice: Voice,
    sine_table: SineTable,
    master_gain: f32,
}

impl Engine {
    /// Construct an uninitialised engine. The sine table is generated
    /// here, once. Call [`Engine::prepare`] before the first
    /// [`Engine::process`] or [`Engine::note_on`].
    pub fn new() -> Self {
        Self {
            sample_rate: 0,
            max_block: 0,
            voice: Voice::new(),
            sine_table: SineTable::new(),
            master_gain: 0.25,
        }
    }

    /// Configure the engine for a given device sample rate and maximum
    /// block size. Safe to call again on sample-rate change.
    pub fn prepare(&mut self, sample_rate: u32, max_block: usize) {
        self.sample_rate = sample_rate;
        self.max_block = max_block;
        self.voice.set_sample_rate(sample_rate as f32);
    }

    /// Adjust the master output gain. Clamped to `[0, 1]`. Default is
    /// `0.25` to leave headroom for future stacked voices.
    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain.clamp(0.0, 1.0);
    }

    /// Trigger a note. No-op if the engine has not been prepared.
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        if self.sample_rate == 0 {
            return;
        }
        self.voice.note_on(note, velocity, self.sample_rate as f32);
    }

    /// Release a note. Matches the active voice's note number; ignored
    /// otherwise (in A2+ the voice handler routes the off to the right
    /// pooled voice).
    pub fn note_off(&mut self, note: u8) {
        self.voice.note_off(note);
    }

    /// Force-release whatever voice is active.
    pub fn all_notes_off(&mut self) {
        self.voice.all_notes_off();
    }

    /// Render audio into the interleaved buffer.
    ///
    /// Mono engine output is broadcast to every channel — stereo
    /// spread / panning lands later (A5/A6 FX bus).
    pub fn process(&mut self, buffer: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buffer.len() / channels;
        for f in 0..frames {
            let sample = self.voice.tick(&self.sine_table) * self.master_gain;
            let base = f * channels;
            for c in 0..channels {
                buffer[base + c] = sample;
            }
        }
    }

    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    #[inline]
    pub fn max_block(&self) -> usize {
        self.max_block
    }
    #[inline]
    pub fn voice_is_active(&self) -> bool {
        self.voice.is_active()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_new_is_idle() {
        let e = Engine::new();
        assert_eq!(e.sample_rate(), 0);
        assert_eq!(e.max_block(), 0);
        assert!(!e.voice_is_active());
    }

    #[test]
    fn prepare_records_rate_and_block() {
        let mut e = Engine::new();
        e.prepare(48_000, 512);
        assert_eq!(e.sample_rate(), 48_000);
        assert_eq!(e.max_block(), 512);
    }

    #[test]
    fn process_writes_silence_without_note() {
        let mut e = Engine::new();
        e.prepare(44_100, 256);
        let mut buf = [0.7f32; 64];
        e.process(&mut buf, 2);
        assert!(buf.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn note_on_makes_engine_audible() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.note_on(69, 100);
        let mut buf = [0.0f32; 1024];
        e.process(&mut buf, 2);
        let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak > 0.0, "expected audio after note_on, got silence");
        // both channels should match (mono fan-out)
        for c in 0..buf.len() / 2 {
            assert_eq!(buf[c * 2], buf[c * 2 + 1]);
        }
    }

    #[test]
    fn all_notes_off_releases_active_voice() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.note_on(60, 100);
        e.all_notes_off();
        // run through worst-case release (default 250 ms ≈ 12k samples)
        let mut buf = [0.0f32; 32_000];
        e.process(&mut buf, 2);
        assert!(!e.voice_is_active());
    }

    #[test]
    fn note_on_before_prepare_is_safe_noop() {
        let mut e = Engine::new();
        e.note_on(69, 100); // engine not prepared yet
        assert!(!e.voice_is_active());
    }
}
