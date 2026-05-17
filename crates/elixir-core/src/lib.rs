//! Elixir DSP engine — core crate.
//!
//! See `ELIXIR-DESIGN.md` and `ELIXIR-PLAN.md` at the repo root for the
//! full architecture and the phased rollout. The engine grows
//! incrementally through phases A0..A6 without breaking the
//! [`Engine::process`] signature locked at A0.
//!
//! Current phase: **A3** — modulation matrix v1. One global LFO + a
//! sparse route table; control-rate evaluation each block; click-free
//! smoothed amounts. Mod-of-mod proven via the `AmpEnv → LfoRate`
//! route.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

pub mod env;
pub mod filter;
pub mod fx;
pub mod lfo;
pub mod modulation;
pub mod osc;
pub mod tables;
pub mod util;
pub mod voice;

use crate::filter::SvfCoeffs;
use crate::fx::{FxSlot, FX_SLOTS};
use crate::lfo::Lfo;
use crate::modulation::{ModMatrix, ModRoute, ModSrc, MAX_GLOBAL_LFOS};
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
    /// Global modulation LFOs. A3 v1 wires up just LFO 0 by default;
    /// the rest can be enabled by future routes / setters.
    lfos: [Lfo; MAX_GLOBAL_LFOS],
    /// Routing table + per-block destination accumulators.
    pub matrix: ModMatrix,
    /// Base voice-filter cutoff in Hz. Modulation adds to this each
    /// block; the sum is clamped to a safe range before coefficients
    /// are derived. A4 default ≈ 8 kHz (wide-open) so an empty preset
    /// sounds essentially unfiltered.
    filter_cutoff_hz: f32,
    /// Voice-filter resonance, `0..1`. `0.0` = flat lowpass; values
    /// approaching `1.0` self-oscillate.
    filter_resonance: f32,
    /// Post-voice FX chain. Slots are processed in order; `FxSlot::Empty`
    /// is skipped. Reorder by swapping slots.
    pub fx_chain: [FxSlot; FX_SLOTS],
    /// Canonical amp-envelope params. Pushed to every voice on every
    /// `set_amp_*` call so the UI can drive ADSR without per-voice
    /// plumbing.
    amp_attack_secs: f32,
    amp_decay_secs: f32,
    amp_sustain: f32,
    amp_release_secs: f32,
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
            lfos: core::array::from_fn(|_| Lfo::new()),
            matrix: ModMatrix::new(),
            filter_cutoff_hz: 8_000.0,
            filter_resonance: 0.0,
            fx_chain: core::array::from_fn(|_| FxSlot::Empty),
            amp_attack_secs: 0.005,
            amp_decay_secs: 0.120,
            amp_sustain: 0.70,
            amp_release_secs: 0.250,
        }
    }

    /// Engine-level amp ADSR setters. Each one is pushed into every
    /// voice's envelope so the UI can drive ADSR without per-voice
    /// plumbing.
    pub fn set_amp_attack_secs(&mut self, s: f32) {
        self.amp_attack_secs = s;
        for v in self.voices.iter_mut() {
            v.set_amp_attack_secs(s);
        }
    }
    pub fn set_amp_decay_secs(&mut self, s: f32) {
        self.amp_decay_secs = s;
        for v in self.voices.iter_mut() {
            v.set_amp_decay_secs(s);
        }
    }
    pub fn set_amp_sustain(&mut self, l: f32) {
        self.amp_sustain = l;
        for v in self.voices.iter_mut() {
            v.set_amp_sustain(l);
        }
    }
    pub fn set_amp_release_secs(&mut self, s: f32) {
        self.amp_release_secs = s;
        for v in self.voices.iter_mut() {
            v.set_amp_release_secs(s);
        }
    }
    pub fn amp_attack_secs(&self) -> f32 {
        self.amp_attack_secs
    }
    pub fn amp_decay_secs(&self) -> f32 {
        self.amp_decay_secs
    }
    pub fn amp_sustain(&self) -> f32 {
        self.amp_sustain
    }
    pub fn amp_release_secs(&self) -> f32 {
        self.amp_release_secs
    }
    pub fn master_gain(&self) -> f32 {
        self.master_gain
    }

    /// Replace the FX in slot `idx`. Returns the previous slot.
    pub fn set_fx_slot(&mut self, idx: usize, slot: FxSlot) -> FxSlot {
        if idx < FX_SLOTS {
            core::mem::replace(&mut self.fx_chain[idx], slot)
        } else {
            slot
        }
    }
    pub fn clear_fx_slot(&mut self, idx: usize) {
        if idx < FX_SLOTS {
            self.fx_chain[idx] = FxSlot::Empty;
        }
    }
    pub fn clear_fx_chain(&mut self) {
        for slot in self.fx_chain.iter_mut() {
            *slot = FxSlot::Empty;
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
        for lfo in self.lfos.iter_mut() {
            lfo.set_sample_rate(sr_f);
        }
    }

    /// Convenience getters/setters for the global LFOs.
    pub fn lfo(&self, idx: usize) -> Option<&Lfo> {
        self.lfos.get(idx)
    }
    pub fn lfo_mut(&mut self, idx: usize) -> Option<&mut Lfo> {
        self.lfos.get_mut(idx)
    }

    /// Add a modulation route. Returns the slot index, or `None` if the
    /// matrix is full.
    pub fn add_mod_route(&mut self, route: ModRoute) -> Option<usize> {
        self.matrix.add_route(route)
    }
    pub fn remove_mod_route(&mut self, idx: usize) {
        self.matrix.remove_route(idx);
    }
    pub fn clear_mod_routes(&mut self) {
        self.matrix.clear();
    }

    /// Adjust the master output gain. Clamped to `[0, 1]`. Default is
    /// `0.25` to leave headroom for stacked voices.
    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain.clamp(0.0, 1.0);
    }

    /// Set the base voice-filter cutoff (Hz). Modulation routes add to
    /// this each block.
    pub fn set_filter_cutoff_hz(&mut self, hz: f32) {
        self.filter_cutoff_hz = hz.clamp(20.0, 22_000.0);
    }
    /// Set the voice-filter resonance, `0..1`.
    pub fn set_filter_resonance(&mut self, r: f32) {
        self.filter_resonance = r.clamp(0.0, 1.0);
    }
    pub fn filter_cutoff_hz(&self) -> f32 {
        self.filter_cutoff_hz
    }
    pub fn filter_resonance(&self) -> f32 {
        self.filter_resonance
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

    /// Average per-voice amp-envelope value. Coarse — used as the
    /// global `ModSrc::AmpEnv` feed in A3 v1.
    fn average_amp_env(&self) -> f32 {
        let mut sum = 0.0f32;
        let mut n = 0u32;
        for v in self.voices.iter() {
            if v.is_active() {
                sum += v.env_value();
                n += 1;
            }
        }
        if n == 0 {
            0.0
        } else {
            sum / n as f32
        }
    }

    /// Render audio into the interleaved buffer.
    ///
    /// Per-block evaluation order matches `ELIXIR-DESIGN.md` §6:
    ///   1. Reset destination accumulators.
    ///   2. Snapshot global source values (envelope average).
    ///   3. Route those into destinations (amount * src).
    ///   4. Apply rate-mod to each LFO, advance LFO phase by `frames`.
    ///   5. Re-evaluate matrix with the now-known LFO values.
    ///   6. Render voices into the buffer; scale by master_gain * (1 +
    ///      master_gain_mod).
    pub fn process(&mut self, buffer: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buffer.len() / channels;
        if frames == 0 {
            return;
        }

        // (1) reset; (2) snapshot non-LFO sources
        self.matrix.reset_destinations();
        let amp_env_avg = self.average_amp_env();

        // (3) first pass: route everything that doesn't depend on LFOs
        //     (LFO values aren't computed yet — feed 0 for them this
        //     pass; they get picked up in pass two).
        self.matrix.route_for_source(|src| match src {
            ModSrc::Constant => 1.0,
            ModSrc::AmpEnv => amp_env_avg,
            ModSrc::Lfo(_) => 0.0,
        });

        // (4) push rate-mod into each LFO and advance one block.
        let mut lfo_values = [0.0f32; MAX_GLOBAL_LFOS];
        for (i, lfo) in self.lfos.iter_mut().enumerate() {
            lfo.set_rate_mod_hz(self.matrix.lfo_rate_mod_hz[i]);
            lfo_values[i] = lfo.tick_block(&self.sine_table, frames);
        }

        // (5) re-eval matrix now that LFO values are known. Pass 1's
        //     LfoRate value has already been applied to the LFO in
        //     step (4), so we can safely reset all destinations and
        //     recompute everything from a single source of truth.
        self.matrix.reset_destinations();
        self.matrix.route_for_source(|src| match src {
            ModSrc::Constant => 1.0,
            ModSrc::AmpEnv => amp_env_avg,
            ModSrc::Lfo(i) => lfo_values.get(i as usize).copied().unwrap_or(0.0),
        });

        // (6) compute filter coefficients from base + cutoff mod
        let effective_cutoff = self.filter_cutoff_hz + self.matrix.filter_cutoff_mod_hz;
        let coeffs = SvfCoeffs::from_params(
            effective_cutoff,
            self.filter_resonance,
            self.sample_rate as f32,
        );

        // (7) render voices at unity into the interleaved buffer; FX
        //     and master gain apply on top.
        for f in 0..frames {
            let mut mix = 0.0f32;
            for v in self.voices.iter_mut() {
                mix += v.tick(&self.sine_table, &coeffs);
            }
            let base = f * channels;
            for c in 0..channels {
                buffer[base + c] = mix;
            }
        }

        // (8) FX chain in declared slot order
        for slot in self.fx_chain.iter_mut() {
            slot.process_inplace(buffer, channels);
        }

        // (9) master gain (post-FX so reverb / delay ride the same fader)
        let effective_gain =
            (self.master_gain * (1.0 + self.matrix.master_gain_mod)).clamp(0.0, 2.0);
        for s in buffer.iter_mut() {
            *s *= effective_gain;
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
    use crate::modulation::ModDest;

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

    // ─── A5 FX chain tests ──────────────────────────────────────────

    #[test]
    fn fx_slot_set_and_clear() {
        use crate::fx::{Drive, FxSlot};
        let mut e = Engine::new();
        e.set_fx_slot(0, FxSlot::Drive(Drive::with_drive(3.0)));
        assert!(matches!(e.fx_chain[0], FxSlot::Drive(_)));
        e.clear_fx_slot(0);
        assert!(matches!(e.fx_chain[0], FxSlot::Empty));
    }

    #[test]
    fn reverb_in_chain_extends_decay_tail() {
        use crate::fx::{FxSlot, Reverb};
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        let mut rv = Reverb::new(48_000.0);
        rv.set_mix(0.9);
        rv.set_decay(0.9);
        e.set_fx_slot(0, FxSlot::Reverb(rv));
        // Short note: 50 ms.
        e.note_on(60, 100);
        let mut quick = vec![0.0f32; 48_000 / 20 * 2];
        e.process(&mut quick, 2);
        e.note_off(60);
        // 300 ms after note-off: original synth would be near-silent
        // (release 250 ms), but reverb tail should still ring.
        let mut tail = vec![0.0f32; 48_000 / 5 * 2];
        e.process(&mut tail, 2);
        let r = rms(&tail);
        assert!(r > 1e-4, "reverb tail too quiet at 300 ms after off: {r}");
    }

    #[test]
    fn drive_in_chain_changes_signal() {
        use crate::fx::{Drive, FxSlot};
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.note_on(69, 127);
        // Render with empty chain
        let mut buf_clean = vec![0.0f32; 4096];
        e.process(&mut buf_clean, 2);
        let rms_clean = rms(&buf_clean);

        // Reset note, add drive
        e.note_off(69);
        let mut silence = vec![0.0f32; 32_000];
        e.process(&mut silence, 2);
        e.set_fx_slot(0, FxSlot::Drive(Drive::with_drive(20.0)));
        e.note_on(69, 127);
        let mut buf_driven = vec![0.0f32; 4096];
        e.process(&mut buf_driven, 2);
        let rms_driven = rms(&buf_driven);

        // Heavily-driven sine should approach a square; RMS goes up.
        assert!(
            rms_driven > rms_clean * 1.2,
            "drive didn't change signal level: clean={rms_clean}, driven={rms_driven}"
        );
    }

    // ─── A4 filter tests ────────────────────────────────────────────

    #[test]
    fn filter_default_is_wide_open() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        assert!(e.filter_cutoff_hz() > 5_000.0);
        assert_eq!(e.filter_resonance(), 0.0);
        // Render a held A4 — should produce a clean tone, not silence.
        e.note_on(69, 100);
        let mut buf = vec![0.0f32; 4096];
        e.process(&mut buf, 2);
        assert!(
            rms(&buf) > 0.01,
            "wide-open filter shouldn't kill the signal"
        );
    }

    #[test]
    fn low_cutoff_attenuates_output() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        // Same note rendered at different cutoffs; RMS should differ.
        let render = |e: &mut Engine, cutoff: f32| -> f32 {
            e.set_filter_cutoff_hz(cutoff);
            e.all_notes_off();
            let mut warm = vec![0.0f32; 8192];
            e.process(&mut warm, 2); // settle filter
            e.note_on(96, 100); // C7 ≈ 2093 Hz
            let mut buf = vec![0.0f32; 16_000];
            e.process(&mut buf, 2);
            rms(&buf)
        };
        let high = render(&mut e, 10_000.0);
        let low = render(&mut e, 400.0);
        assert!(
            low < high * 0.5,
            "low cutoff should attenuate C7 vs open: high={high}, low={low}"
        );
    }

    #[test]
    fn lfo_modulating_filter_cutoff_changes_amplitude_over_time() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        // Park the cutoff near the signal frequency and modulate it
        // ±1200 Hz with an LFO. The amplitude through the filter should
        // swing as the cutoff sweeps past the signal.
        e.set_filter_cutoff_hz(1_500.0);
        e.lfo_mut(0).unwrap().set_rate_hz(4.0);
        e.add_mod_route(ModRoute::new(
            ModSrc::Lfo(0),
            ModDest::FilterCutoff,
            1_500.0,
        ))
        .unwrap();
        e.note_on(81, 100); // A5 = 880 Hz
        let mut chunk_rms = Vec::with_capacity(24);
        for _ in 0..24 {
            let mut buf = vec![0.0f32; 2000];
            e.process(&mut buf, 2);
            chunk_rms.push(rms(&buf));
        }
        let min = chunk_rms.iter().fold(f32::INFINITY, |a, b| a.min(*b));
        let max = chunk_rms.iter().fold(0.0f32, |a, b| a.max(*b));
        assert!(
            max > min * 1.5,
            "filter sweep didn't change amplitude: min={min}, max={max}"
        );
    }

    // ─── A3 modulation matrix tests ─────────────────────────────────

    #[test]
    fn lfo_modulating_master_gain_produces_tremolo() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.lfo_mut(0).unwrap().set_rate_hz(8.0); // 8 Hz tremolo
        e.add_mod_route(ModRoute::new(ModSrc::Lfo(0), ModDest::MasterGain, 0.5))
            .unwrap();
        e.note_on(69, 100);

        // Render ~1 second in chunks; collect per-chunk RMS.
        let mut chunk_rms = Vec::with_capacity(48);
        for _ in 0..48 {
            let mut buf = vec![0.0f32; 1000];
            e.process(&mut buf, 2);
            chunk_rms.push(rms(&buf));
        }
        let min = chunk_rms.iter().fold(f32::INFINITY, |a, b| a.min(*b));
        let max = chunk_rms.iter().fold(0.0f32, |a, b| a.max(*b));
        // Tremolo: peak/trough ratio should be visible.
        assert!(max > min * 1.5, "tremolo not audible: min={min}, max={max}");
    }

    #[test]
    fn amp_env_modulates_lfo_rate() {
        // Verify ModDest::LfoRate gets a non-zero contribution from
        // ModSrc::AmpEnv when a route is wired.
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.add_mod_route(ModRoute::new(ModSrc::AmpEnv, ModDest::LfoRate(0), 12.0))
            .unwrap();
        e.note_on(60, 100);
        // Drive a few blocks so the envelope ramps up into Sustain.
        let mut buf = vec![0.0f32; 4096];
        for _ in 0..6 {
            e.process(&mut buf, 2);
        }
        // Sustain level is 0.7 by default → expected rate mod = 0.7 * 12 = 8.4 Hz
        let mod_hz = e.matrix.lfo_rate_mod_hz[0];
        assert!(
            mod_hz > 5.0 && mod_hz < 9.0,
            "expected LFO rate mod near 8.4 Hz, got {mod_hz}"
        );
    }

    #[test]
    fn matrix_clear_removes_all_routes() {
        use crate::modulation::MAX_ROUTES;
        let mut e = Engine::new();
        e.add_mod_route(ModRoute::new(ModSrc::Constant, ModDest::MasterGain, 0.1))
            .unwrap();
        e.add_mod_route(ModRoute::new(ModSrc::Lfo(0), ModDest::MasterGain, 0.2))
            .unwrap();
        assert_eq!(e.matrix.route_count(), 2);
        e.clear_mod_routes();
        assert_eq!(e.matrix.route_count(), 0);
        // matrix should still accept new routes after clear
        for _ in 0..MAX_ROUTES {
            assert!(e
                .add_mod_route(ModRoute::new(ModSrc::Constant, ModDest::MasterGain, 0.0))
                .is_some());
        }
        assert!(e
            .add_mod_route(ModRoute::new(ModSrc::Constant, ModDest::MasterGain, 0.0))
            .is_none());
    }

    #[test]
    fn no_routes_keeps_master_gain_steady() {
        // Sanity: A3 must not change A1/A2 baseline when no routes exist.
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.note_on(69, 100);
        let mut buf = vec![0.0f32; 4096];
        e.process(&mut buf, 2);
        assert!(rms(&buf) > 0.001);
        assert_eq!(e.matrix.master_gain_mod, 0.0);
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
