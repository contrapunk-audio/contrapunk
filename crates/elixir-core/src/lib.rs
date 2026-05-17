//! Elixir DSP engine — core crate.
//!
//! See `ELIXIR-DESIGN.md` and `ELIXIR-PLAN.md` at the repo root for the
//! full architecture and the phased rollout. The engine grows
//! incrementally through phases A0..A6 without breaking the
//! [`Engine::process`] signature locked at A0.
//!
//! Current phase: **A2** — 16-voice polyphony, voice stealing (Newest
//! priority), kill ramps, sustain pedal.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

pub mod env;
pub mod osc;
pub mod tables;
pub mod util;
pub mod voice;

use crate::tables::SineTable;
use crate::voice::Voice;

/// Voice pool capacity. `MAX_POLYPHONY` voices count as "live"; the
/// extra slots hold killing voices that are still ringing out their
/// 5 ms kill ramp. Sized so a steal can re-trigger immediately into a
/// fresh slot.
pub const MAX_POLYPHONY: usize = 16;
pub const PARALLEL_VOICES: usize = 4;
pub const MAX_VOICES: usize = MAX_POLYPHONY + PARALLEL_VOICES;

/// The top-level Elixir engine.
///
/// One instance per audio thread. Owns the voice pool + a shared sine
/// table. A1 was single-voice; A2 grows to a `[Voice; MAX_VOICES]`
/// pool. A future A2 follow-up will SIMD-pack voices into
/// `AggregateVoice` groups (two voices per `f32x8` lane).
pub struct Engine {
    sample_rate: u32,
    max_block: usize,
    voices: [Voice; MAX_VOICES],
    sine_table: SineTable,
    master_gain: f32,
    sustain_pedal: bool,
    note_counter: u64,
}

impl Engine {
    /// Construct an uninitialised engine. The sine table is generated
    /// here, once. Call [`Engine::prepare`] before the first
    /// [`Engine::process`] or [`Engine::note_on`].
    pub fn new() -> Self {
        Self {
            sample_rate: 0,
            max_block: 0,
            voices: core::array::from_fn(|_| Voice::new()),
            sine_table: SineTable::new(),
            master_gain: 0.25,
            sustain_pedal: false,
            note_counter: 0,
        }
    }

    /// Configure the engine for a given device sample rate and maximum
    /// block size. Safe to call again on sample-rate change.
    pub fn prepare(&mut self, sample_rate: u32, max_block: usize) {
        self.sample_rate = sample_rate;
        self.max_block = max_block;
        let sr_f = sample_rate as f32;
        for v in self.voices.iter_mut() {
            v.set_sample_rate(sr_f);
        }
    }

    /// Adjust the master output gain. Clamped to `[0, 1]`. Default is
    /// `0.25` to leave headroom for stacked voices.
    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain.clamp(0.0, 1.0);
    }

    /// Engage / release the sustain pedal (CC 64). When released, any
    /// voices that were sustain-held drop into release.
    pub fn set_sustain_pedal(&mut self, on: bool) {
        let was_on = self.sustain_pedal;
        self.sustain_pedal = on;
        if was_on && !on {
            for v in self.voices.iter_mut() {
                v.release_sustain();
            }
        }
    }

    /// Trigger a note. No-op if the engine has not been prepared.
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        if self.sample_rate == 0 {
            return;
        }
        let sr = self.sample_rate as f32;
        let age = self.note_counter;
        self.note_counter = self.note_counter.wrapping_add(1);

        // 1. Retrigger: same note already live → reuse slot, no steal.
        for v in self.voices.iter_mut() {
            if v.is_playing_note(note) {
                v.note_on(note, velocity, sr, age);
                return;
            }
        }

        // 2. If we're at the live-polyphony cap, force-kill the oldest
        //    live voice so a slot opens up.
        let live_count = self.voices.iter().filter(|v| v.is_live()).count();
        if live_count >= MAX_POLYPHONY {
            let mut oldest_idx = 0usize;
            let mut oldest_age = u64::MAX;
            for (i, v) in self.voices.iter().enumerate() {
                if v.is_live() && v.age() < oldest_age {
                    oldest_age = v.age();
                    oldest_idx = i;
                }
            }
            self.voices[oldest_idx].kill();
        }

        // 3. Find a non-live slot. Prefer fully inactive over killing.
        let mut inactive_idx: Option<usize> = None;
        let mut killing_idx: Option<usize> = None;
        for (i, v) in self.voices.iter().enumerate() {
            if !v.is_active() {
                inactive_idx = Some(i);
                break;
            } else if v.is_killing() && killing_idx.is_none() {
                killing_idx = Some(i);
            }
        }
        if let Some(i) = inactive_idx.or(killing_idx) {
            self.voices[i].note_on(note, velocity, sr, age);
            return;
        }

        // 4. Fallback (shouldn't reach): every slot is live. Steal the
        //    oldest outright.
        let mut oldest_idx = 0usize;
        let mut oldest_age = u64::MAX;
        for (i, v) in self.voices.iter().enumerate() {
            if v.age() < oldest_age {
                oldest_age = v.age();
                oldest_idx = i;
            }
        }
        self.voices[oldest_idx].note_on(note, velocity, sr, age);
    }

    /// Release a note. If sustain is down, the matching voice goes
    /// `sustained` instead of `released`.
    pub fn note_off(&mut self, note: u8) {
        let pedal = self.sustain_pedal;
        for v in self.voices.iter_mut() {
            v.note_off_or_sustain(note, pedal);
        }
    }

    /// Force-release every voice. Drops the sustain-pedal state too.
    pub fn all_notes_off(&mut self) {
        for v in self.voices.iter_mut() {
            v.all_notes_off();
        }
        self.sustain_pedal = false;
    }

    /// Render audio into the interleaved buffer.
    pub fn process(&mut self, buffer: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buffer.len() / channels;
        for f in 0..frames {
            let mut mix = 0.0f32;
            for v in self.voices.iter_mut() {
                mix += v.tick(&self.sine_table);
            }
            mix *= self.master_gain;
            let base = f * channels;
            for c in 0..channels {
                buffer[base + c] = mix;
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
    pub fn live_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_live()).count()
    }
    #[inline]
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }
    #[inline]
    pub fn sustain_pedal(&self) -> bool {
        self.sustain_pedal
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

    fn rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let s: f64 = samples.iter().map(|x| (*x as f64).powi(2)).sum();
        (s / samples.len() as f64).sqrt() as f32
    }

    #[test]
    fn engine_new_is_idle() {
        let e = Engine::new();
        assert_eq!(e.sample_rate(), 0);
        assert_eq!(e.max_block(), 0);
        assert_eq!(e.active_voice_count(), 0);
        assert_eq!(e.live_voice_count(), 0);
        assert!(!e.sustain_pedal());
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
        for c in 0..buf.len() / 2 {
            assert_eq!(buf[c * 2], buf[c * 2 + 1]);
        }
    }

    #[test]
    fn all_notes_off_releases_active_voices() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.note_on(60, 100);
        e.note_on(64, 100);
        e.note_on(67, 100);
        assert_eq!(e.live_voice_count(), 3);
        e.all_notes_off();
        let mut buf = [0.0f32; 32_000];
        e.process(&mut buf, 2);
        assert_eq!(e.active_voice_count(), 0);
    }

    #[test]
    fn note_on_before_prepare_is_safe_noop() {
        let mut e = Engine::new();
        e.note_on(69, 100);
        assert_eq!(e.live_voice_count(), 0);
    }

    #[test]
    fn sixteen_note_chord_plays_polyphonically() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        // C major scale spread across two octaves = 16 notes
        let notes: [u8; 16] = [
            48, 50, 52, 53, 55, 57, 59, 60, 62, 64, 65, 67, 69, 71, 72, 74,
        ];
        for &n in &notes {
            e.note_on(n, 90);
        }
        assert_eq!(e.live_voice_count(), MAX_POLYPHONY);
        let mut buf = vec![0.0f32; 4096];
        e.process(&mut buf, 2);
        // Mix should be visibly louder than a single voice.
        assert!(rms(&buf) > 0.01, "16-note chord rendered too quietly");
    }

    #[test]
    fn seventeenth_note_steals_oldest_voice() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        for n in 48u8..(48 + MAX_POLYPHONY as u8) {
            e.note_on(n, 90);
        }
        assert_eq!(e.live_voice_count(), MAX_POLYPHONY);
        // Trigger one more — the oldest (note 48) should be killed.
        e.note_on(80, 90);
        // Still 16 live voices total (the new one replaces one stolen one).
        assert_eq!(e.live_voice_count(), MAX_POLYPHONY);
        // 17 active slots though: 16 live + 1 killing.
        assert_eq!(e.active_voice_count(), MAX_POLYPHONY + 1);
    }

    #[test]
    fn retrigger_same_note_does_not_consume_extra_slot() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.note_on(60, 100);
        assert_eq!(e.live_voice_count(), 1);
        e.note_on(60, 100); // same note again
        assert_eq!(e.live_voice_count(), 1);
    }

    #[test]
    fn sustain_pedal_holds_note_through_note_off() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.set_sustain_pedal(true);
        e.note_on(60, 100);
        e.note_off(60);
        // Voice should still be active (held by sustain).
        assert_eq!(e.live_voice_count(), 1);
        // Render a chunk; voice keeps playing.
        let mut buf = vec![0.0f32; 4096];
        e.process(&mut buf, 2);
        assert!(rms(&buf) > 0.001);
        // Release pedal — voice now releases.
        e.set_sustain_pedal(false);
        let mut buf2 = vec![0.0f32; 32_000];
        e.process(&mut buf2, 2);
        assert_eq!(e.active_voice_count(), 0);
    }
}
