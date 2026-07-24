//! Elixir DSP engine — core crate.
//!
//! See `ELIXIR-DESIGN.md` and `ELIXIR-PLAN.md` at the repo root for the
//! full architecture and the phased rollout. The engine grows
//! incrementally through phases A0..A6 without breaking the
//! [`Engine::process`] signature locked at A0.
//!
//! Current implementation checkpoint: **A6 audit**. A0-A5 foundations
//! are in place, and the A6 public surface is implemented for spectral
//! morph selectors, phase-distortion modes, unison styles, expanded
//! filter models, the 8-slot FX chain, and modulation/dynamics FX. This
//! crate intentionally keeps the same [`Engine::process`] contract while
//! QA hardens each completed feature.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

#[cfg(test)]
#[global_allocator]
static TEST_ALLOCATOR: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;

pub mod env;
pub mod filter;
pub mod fx;
pub mod lfo;
pub mod modulation;
pub mod osc;
pub mod tables;
pub mod util;
pub mod voice;

use crate::filter::{FilterKind, FilterParams};
use crate::fx::{FxSlot, FX_SLOTS};
use crate::lfo::Lfo;
use crate::modulation::{ModMatrix, ModRoute, ModSrc, MAX_GLOBAL_LFOS};
use crate::osc::{OscParams, PhaseDistortionMode, SpectralMorph, UnisonStyle};
use crate::tables::SineTable;
use crate::voice::Voice;

/// Stable caller-owned identity for one sounding voice.
///
/// The high bit is reserved for the temporary MIDI-note compatibility
/// wrappers; canonical callers should keep it clear.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VoiceId(u64);

impl VoiceId {
    pub const INVALID: Self = Self(u64::MAX);
    const LEGACY_MIDI_PREFIX: u64 = 1 << 63;

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    const fn from_midi_note(note: u8) -> Self {
        Self(Self::LEGACY_MIDI_PREFIX | note as u64)
    }
}

/// Contrapunk performance role retained with each voice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum VoiceRole {
    #[default]
    Input = 0,
    Harmony = 1,
    Canon = 2,
    Counterpoint = 3,
}

impl VoiceRole {
    pub const ALL: [Self; 4] = [Self::Input, Self::Harmony, Self::Canon, Self::Counterpoint];

    const fn index(self) -> usize {
        self as usize
    }
}

/// Canonical host-neutral voice event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoiceEvent {
    NoteOn {
        voice_id: VoiceId,
        role: VoiceRole,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
    },
    NoteOff {
        voice_id: VoiceId,
    },
    Panic,
}

/// Voice pool capacity. `MAX_POLYPHONY` voices count as "live"; the
/// extra slots hold killing voices that are still ringing out their
/// 5 ms kill ramp. Sized so a steal can re-trigger immediately into a
/// fresh slot.
pub const MAX_POLYPHONY: usize = 16;
pub const PARALLEL_VOICES: usize = 4;
pub const MAX_VOICES: usize = MAX_POLYPHONY + PARALLEL_VOICES;

/// The top-level Elixir engine.
///
/// One instance per audio thread. Owns the voice pool, shared sine table,
/// modulation matrix, oscillator controls, voice filter controls, and FX
/// chain. The current implementation is scalar and allocation-free while
/// processing; future SIMD packing can happen behind the same public API.
pub struct Engine {
    sample_rate: u32,
    max_block: usize,
    voices: [Voice; MAX_VOICES],
    sine_table: SineTable,
    master_gain: f32,
    role_gains: [f32; VoiceRole::ALL.len()],
    sustain_pedal: bool,
    note_counter: u64,
    /// Global modulation LFOs. A3 v1 wires up just LFO 0 by default;
    /// the rest can be enabled by future routes / setters.
    lfos: [Lfo; MAX_GLOBAL_LFOS],
    /// Routing table + per-block destination accumulators.
    pub matrix: ModMatrix,
    /// Engine-wide oscillator parameters (A6): spectral morph, phase
    /// distortion, and unison stack. Defaults preserve the A1 sine path.
    osc_params: OscParams,
    /// Base voice-filter cutoff in Hz. Modulation adds to this each
    /// block; the sum is clamped to a safe range before coefficients
    /// are derived. A4 default ≈ 8 kHz (wide-open) so an empty preset
    /// sounds essentially unfiltered.
    filter_cutoff_hz: f32,
    /// Voice-filter resonance, `0..1`. `0.0` = flat lowpass; values
    /// approaching `1.0` self-oscillate.
    filter_resonance: f32,
    /// A6 filter model and color controls.
    filter_kind: FilterKind,
    filter_drive: f32,
    filter_gain: f32,
    filter_morph_x: f32,
    filter_morph_y: f32,
    /// Post-voice FX chain. Slots are processed in order; `FxSlot::Empty`
    /// is skipped. Reorder by swapping slots.
    pub fx_chain: [FxSlot; FX_SLOTS],
    fx_enabled: [bool; FX_SLOTS],
    /// A non-finite FX result quarantines the chain without dropping its
    /// preallocated state on the audio thread.
    fx_quarantined: bool,
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
            role_gains: [1.0; VoiceRole::ALL.len()],
            sustain_pedal: false,
            note_counter: 0,
            lfos: core::array::from_fn(|_| Lfo::new()),
            matrix: ModMatrix::new(),
            osc_params: OscParams::default(),
            filter_cutoff_hz: 8_000.0,
            filter_resonance: 0.0,
            filter_kind: FilterKind::DigitalSvf,
            filter_drive: 1.0,
            filter_gain: 1.0,
            filter_morph_x: 0.0,
            filter_morph_y: 0.0,
            fx_chain: core::array::from_fn(|_| FxSlot::Empty),
            fx_enabled: [true; FX_SLOTS],
            fx_quarantined: false,
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
        if !s.is_finite() {
            return;
        }
        let s = s.max(0.001);
        self.amp_attack_secs = s;
        for v in self.voices.iter_mut() {
            v.set_amp_attack_secs(s);
        }
    }
    pub fn set_amp_decay_secs(&mut self, s: f32) {
        if !s.is_finite() {
            return;
        }
        let s = s.max(0.001);
        self.amp_decay_secs = s;
        for v in self.voices.iter_mut() {
            v.set_amp_decay_secs(s);
        }
    }
    pub fn set_amp_sustain(&mut self, l: f32) {
        if !l.is_finite() {
            return;
        }
        let l = l.clamp(0.0, 1.0);
        self.amp_sustain = l;
        for v in self.voices.iter_mut() {
            v.set_amp_sustain(l);
        }
    }
    pub fn set_amp_release_secs(&mut self, s: f32) {
        if !s.is_finite() {
            return;
        }
        let s = s.max(0.001);
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
    pub fn set_role_gain(&mut self, role: VoiceRole, gain: f32) {
        if gain.is_finite() {
            self.role_gains[role.index()] = gain.clamp(0.0, 1.0);
        }
    }
    pub fn role_gains(&self) -> [f32; VoiceRole::ALL.len()] {
        self.role_gains
    }
    pub fn role_gain(&self, role: VoiceRole) -> f32 {
        self.role_gains[role.index()]
    }

    /// A6 oscillator params. Defaults are passthrough/single-voice sine,
    /// preserving A1-A5 render behavior unless callers opt in.
    pub fn osc_params(&self) -> OscParams {
        self.osc_params
    }
    pub fn set_spectral_morph(&mut self, morph: SpectralMorph) {
        self.osc_params.spectral_morph = morph;
    }
    pub fn set_morph_amount(&mut self, amount: f32) {
        if amount.is_finite() {
            self.osc_params.morph_amount = amount.clamp(0.0, 1.0);
        }
    }
    pub fn set_phase_distortion(&mut self, mode: PhaseDistortionMode) {
        self.osc_params.phase_distortion = mode;
    }
    pub fn set_phase_amount(&mut self, amount: f32) {
        if amount.is_finite() {
            self.osc_params.phase_amount = amount.clamp(0.0, 1.0);
        }
    }
    pub fn set_unison_voices(&mut self, voices: u8) {
        self.osc_params.unison_voices = voices.clamp(1, crate::osc::MAX_UNISON as u8);
    }
    pub fn set_unison_detune_cents(&mut self, cents: f32) {
        if cents.is_finite() {
            self.osc_params.unison_detune_cents = cents.clamp(0.0, 1200.0);
        }
    }
    pub fn set_unison_style(&mut self, style: UnisonStyle) {
        self.osc_params.unison_style = style;
    }

    /// Replace the FX in slot `idx`. Returns the previous slot.
    pub fn set_fx_slot(&mut self, idx: usize, slot: FxSlot) -> FxSlot {
        if idx < FX_SLOTS {
            self.fx_enabled[idx] = true;
            core::mem::replace(&mut self.fx_chain[idx], slot)
        } else {
            slot
        }
    }
    pub fn set_fx_enabled(&mut self, idx: usize, enabled: bool) {
        if idx < FX_SLOTS {
            self.fx_enabled[idx] = enabled;
        }
    }
    pub fn fx_enabled(&self, idx: usize) -> bool {
        self.fx_enabled.get(idx).copied().unwrap_or(false)
    }
    pub fn clear_fx_slot(&mut self, idx: usize) {
        if idx < FX_SLOTS {
            self.fx_chain[idx] = FxSlot::Empty;
            self.fx_enabled[idx] = false;
        }
    }
    pub fn clear_fx_chain(&mut self) {
        for (slot, enabled) in self.fx_chain.iter_mut().zip(self.fx_enabled.iter_mut()) {
            *slot = FxSlot::Empty;
            *enabled = false;
        }
        self.fx_quarantined = false;
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
        if gain.is_finite() {
            self.master_gain = gain.clamp(0.0, 1.0);
        }
    }

    /// Set the base voice-filter cutoff (Hz). Modulation routes add to
    /// this each block.
    pub fn set_filter_cutoff_hz(&mut self, hz: f32) {
        if hz.is_finite() {
            self.filter_cutoff_hz = hz.clamp(20.0, 22_000.0);
        }
    }
    /// Set the voice-filter resonance, `0..1`.
    pub fn set_filter_resonance(&mut self, r: f32) {
        if r.is_finite() {
            self.filter_resonance = r.clamp(0.0, 1.0);
        }
    }
    pub fn set_filter_kind(&mut self, kind: FilterKind) {
        self.filter_kind = kind;
        for v in self.voices.iter_mut() {
            v.set_filter_kind(kind);
        }
    }
    pub fn set_filter_drive(&mut self, drive: f32) {
        if drive.is_finite() {
            self.filter_drive = drive.clamp(0.1, 32.0);
        }
    }
    pub fn set_filter_gain(&mut self, gain: f32) {
        if gain.is_finite() {
            self.filter_gain = gain.clamp(0.0, 4.0);
        }
    }
    pub fn set_filter_morph(&mut self, x: f32, y: f32) {
        if x.is_finite() && y.is_finite() {
            self.filter_morph_x = x.clamp(0.0, 1.0);
            self.filter_morph_y = y.clamp(0.0, 1.0);
        }
    }
    pub fn filter_cutoff_hz(&self) -> f32 {
        self.filter_cutoff_hz
    }
    pub fn filter_resonance(&self) -> f32 {
        self.filter_resonance
    }
    pub fn filter_kind(&self) -> FilterKind {
        self.filter_kind
    }
    pub fn filter_drive(&self) -> f32 {
        self.filter_drive
    }
    pub fn filter_gain(&self) -> f32 {
        self.filter_gain
    }
    pub fn filter_morph(&self) -> (f32, f32) {
        (self.filter_morph_x, self.filter_morph_y)
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

    /// Apply one canonical voice event. Invalid/non-positive frequencies
    /// and events received before `prepare` are safe no-ops.
    pub fn handle_voice_event(&mut self, event: VoiceEvent) {
        match event {
            VoiceEvent::NoteOn {
                voice_id,
                role,
                midi_anchor,
                frequency_hz,
                velocity,
            } => self.start_voice(voice_id, role, midi_anchor, frequency_hz, velocity),
            VoiceEvent::NoteOff { voice_id } => self.release_voice(voice_id),
            VoiceEvent::Panic => self.panic(),
        }
    }

    fn start_voice(
        &mut self,
        voice_id: VoiceId,
        role: VoiceRole,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
    ) {
        if self.sample_rate == 0
            || voice_id == VoiceId::INVALID
            || midi_anchor >= 128
            || !frequency_hz.is_finite()
            || frequency_hz <= 0.0
        {
            return;
        }
        let sr = self.sample_rate as f32;
        let age = self.note_counter;
        self.note_counter = self.note_counter.wrapping_add(1);

        // Retrigger the caller-owned voice without consuming a new slot.
        for v in self.voices.iter_mut() {
            if v.has_voice_id(voice_id) {
                v.note_on(voice_id, role, midi_anchor, frequency_hz, velocity, sr, age);
                return;
            }
        }

        // If we're at the live-polyphony cap, force-kill the oldest live
        // voice so a slot opens immediately for the new identity.
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

        // Prefer a fully inactive slot over a voice already on its bounded
        // kill ramp.
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
            self.voices[i].note_on(voice_id, role, midi_anchor, frequency_hz, velocity, sr, age);
            return;
        }

        // Defensive fallback: steal the oldest slot outright.
        let mut oldest_idx = 0usize;
        let mut oldest_age = u64::MAX;
        for (i, v) in self.voices.iter().enumerate() {
            if v.age() < oldest_age {
                oldest_age = v.age();
                oldest_idx = i;
            }
        }
        self.voices[oldest_idx].note_on(
            voice_id,
            role,
            midi_anchor,
            frequency_hz,
            velocity,
            sr,
            age,
        );
    }

    fn release_voice(&mut self, voice_id: VoiceId) {
        let pedal = self.sustain_pedal;
        for v in self.voices.iter_mut() {
            v.note_off_or_sustain(voice_id, pedal);
        }
    }

    /// Temporary 12-TET compatibility wrapper for MIDI-note callers.
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        self.start_voice(
            VoiceId::from_midi_note(note),
            VoiceRole::Input,
            note,
            crate::util::midi_to_freq(note),
            velocity,
        );
    }

    /// Temporary 12-TET compatibility wrapper for MIDI-note callers.
    pub fn note_off(&mut self, note: u8) {
        self.release_voice(VoiceId::from_midi_note(note));
    }

    /// Release every voice with the configured envelope tail.
    pub fn all_notes_off(&mut self) {
        for v in self.voices.iter_mut() {
            v.all_notes_off();
        }
        self.sustain_pedal = false;
    }

    /// Fast bounded panic ramp for every voice and pedal state.
    pub fn panic(&mut self) {
        for v in self.voices.iter_mut() {
            v.kill();
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

        // (6) compute filter params from base + cutoff mod
        let effective_cutoff = self.filter_cutoff_hz + self.matrix.filter_cutoff_mod_hz;
        let filter_params = FilterParams {
            kind: self.filter_kind,
            cutoff_hz: effective_cutoff,
            resonance: self.filter_resonance,
            drive: self.filter_drive,
            gain: self.filter_gain,
            morph_x: self.filter_morph_x,
            morph_y: self.filter_morph_y,
            sample_rate: self.sample_rate as f32,
        };

        // (6b) prepare filter coefficients ONCE for the block. The hot
        //      loop below is now `tanf`-free for every voice.
        let filter_coeffs = filter_params.prepare_coeffs();

        // (7) render voices with their retained performance-role gain;
        //     FX and master gain apply on top.
        let role_gains = self.role_gains;
        for f in 0..frames {
            let mut mix = 0.0f32;
            for v in self.voices.iter_mut() {
                mix +=
                    v.tick_with_filter_coeffs(&self.sine_table, &filter_coeffs, &self.osc_params)
                        * role_gains[v.role().index()];
            }
            let base = f * channels;
            for c in 0..channels {
                buffer[base + c] = mix;
            }
        }

        // (8) FX chain in declared slot order. A poisoned chain remains
        // allocated but bypassed until a non-audio control clears it.
        if !self.fx_quarantined {
            for (slot, enabled) in self.fx_chain.iter_mut().zip(self.fx_enabled) {
                if enabled {
                    slot.process_inplace(buffer, channels);
                }
            }
        }

        // (9) master gain (post-FX so reverb / delay ride the same fader)
        let effective_gain =
            (self.master_gain * (1.0 + self.matrix.master_gain_mod)).clamp(0.0, 2.0);
        let mut poisoned = false;
        for s in buffer.iter_mut() {
            *s *= effective_gain;
            if !s.is_finite() {
                *s = 0.0;
                poisoned = true;
            }
        }
        if poisoned {
            self.fx_quarantined = true;
            self.panic();
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
    fn non_finite_engine_controls_preserve_last_valid_state() {
        let mut e = Engine::new();
        e.set_amp_attack_secs(0.2);
        e.set_amp_decay_secs(0.3);
        e.set_amp_sustain(0.4);
        e.set_amp_release_secs(0.5);
        e.set_master_gain(0.6);
        e.set_morph_amount(0.7);
        e.set_phase_amount(0.8);
        e.set_unison_detune_cents(9.0);
        e.set_filter_cutoff_hz(1_000.0);
        e.set_filter_resonance(0.2);
        e.set_filter_drive(3.0);
        e.set_filter_gain(1.5);
        e.set_filter_morph(0.25, 0.75);

        e.set_amp_attack_secs(f32::NAN);
        e.set_amp_decay_secs(f32::INFINITY);
        e.set_amp_sustain(f32::NEG_INFINITY);
        e.set_amp_release_secs(f32::NAN);
        e.set_master_gain(f32::NAN);
        e.set_morph_amount(f32::INFINITY);
        e.set_phase_amount(f32::NEG_INFINITY);
        e.set_unison_detune_cents(f32::NAN);
        e.set_filter_cutoff_hz(f32::INFINITY);
        e.set_filter_resonance(f32::NAN);
        e.set_filter_drive(f32::NEG_INFINITY);
        e.set_filter_gain(f32::NAN);
        e.set_filter_morph(f32::NAN, 0.5);

        assert_eq!(e.amp_attack_secs(), 0.2);
        assert_eq!(e.amp_decay_secs(), 0.3);
        assert_eq!(e.amp_sustain(), 0.4);
        assert_eq!(e.amp_release_secs(), 0.5);
        assert_eq!(e.master_gain(), 0.6);
        assert_eq!(e.osc_params().morph_amount, 0.7);
        assert_eq!(e.osc_params().phase_amount, 0.8);
        assert_eq!(e.osc_params().unison_detune_cents, 9.0);
        assert_eq!(e.filter_cutoff_hz(), 1_000.0);
        assert_eq!(e.filter_resonance(), 0.2);
        assert_eq!(e.filter_drive(), 3.0);
        assert_eq!(e.filter_gain(), 1.5);
        assert_eq!(e.filter_morph(), (0.25, 0.75));
    }

    #[test]
    fn non_finite_output_is_silenced_and_poisoned_state_is_quarantined() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.note_on(69, 100);
        e.master_gain = f32::NAN;

        let mut poisoned = [1.0; 1024];
        e.process(&mut poisoned, 2);
        assert!(poisoned.iter().all(|sample| *sample == 0.0));
        assert!(e.fx_quarantined);
        assert_eq!(e.live_voice_count(), 0);

        e.master_gain = 0.25;
        e.note_on(71, 100);
        let mut recovered = [0.0; 1024];
        e.process(&mut recovered, 2);
        assert!(recovered.iter().all(|sample| sample.is_finite()));
        assert!(recovered.iter().any(|sample| sample.abs() > 1.0e-6));
    }

    #[test]
    fn steady_state_voice_and_fx_processing_allocates_nothing() {
        use crate::fx::{Chorus, Compressor, Delay, Drive, FdnReverb, Flanger, Phaser, Reverb};

        let mut e = Engine::new();
        e.prepare(48_000, 256);
        let slots = [
            FxSlot::Drive(Drive::new()),
            FxSlot::Delay(Delay::new(1024)),
            FxSlot::Reverb(Reverb::new(48_000.0)),
            FxSlot::FdnReverb(FdnReverb::new(48_000.0)),
            FxSlot::Chorus(Chorus::new(48_000.0)),
            FxSlot::Flanger(Flanger::new(48_000.0)),
            FxSlot::Phaser(Phaser::new(48_000.0)),
            FxSlot::Compressor(Compressor::new(48_000.0)),
        ];
        for (index, slot) in slots.into_iter().enumerate() {
            e.set_fx_slot(index, slot);
        }
        let mut audio = [0.0; 512];

        assert_no_alloc::assert_no_alloc(|| {
            e.handle_voice_event(note_on_event(1, VoiceRole::Input, 440.0));
            e.process(&mut audio, 2);
            e.handle_voice_event(VoiceEvent::NoteOff {
                voice_id: VoiceId::new(1),
            });
            e.process(&mut audio, 2);
            e.handle_voice_event(VoiceEvent::Panic);
            e.process(&mut audio, 2);
        });
        assert!(audio.iter().all(|sample| sample.is_finite()));
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
        e.note_on(60, 100); // same compatibility-wrapper identity
        assert_eq!(e.live_voice_count(), 1);
    }

    fn note_on_event(voice_id: u64, role: VoiceRole, frequency_hz: f32) -> VoiceEvent {
        VoiceEvent::NoteOn {
            voice_id: VoiceId::new(voice_id),
            role,
            midi_anchor: 69,
            frequency_hz,
            velocity: 100,
        }
    }

    #[test]
    fn canonical_note_on_rejects_invalid_anchor_and_frequency() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.handle_voice_event(VoiceEvent::NoteOn {
            voice_id: VoiceId::new(1),
            role: VoiceRole::Input,
            midi_anchor: 128,
            frequency_hz: 440.0,
            velocity: 100,
        });
        e.handle_voice_event(VoiceEvent::NoteOn {
            voice_id: VoiceId::new(2),
            role: VoiceRole::Input,
            midi_anchor: 69,
            frequency_hz: f32::NAN,
            velocity: 100,
        });
        assert_eq!(e.live_voice_count(), 0);
    }

    #[test]
    fn canonical_same_anchor_voices_retain_independent_identity_frequency_and_role() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.handle_voice_event(note_on_event(1, VoiceRole::Input, 440.0));
        e.handle_voice_event(note_on_event(2, VoiceRole::Harmony, 442.0));

        assert_eq!(e.live_voice_count(), 2);
        assert!(e.voices.iter().any(|v| {
            v.owns_voice_id(VoiceId::new(1))
                && v.role() == VoiceRole::Input
                && v.midi_anchor() == 69
                && v.frequency_hz() == 440.0
        }));
        assert!(e.voices.iter().any(|v| {
            v.owns_voice_id(VoiceId::new(2))
                && v.role() == VoiceRole::Harmony
                && v.midi_anchor() == 69
                && v.frequency_hz() == 442.0
        }));
    }

    #[test]
    fn retained_role_gain_is_applied_selectively() {
        let mut mixed = Engine::new();
        mixed.prepare(48_000, 256);
        mixed.set_role_gain(VoiceRole::Input, 0.0);
        mixed.handle_voice_event(note_on_event(1, VoiceRole::Input, 440.0));
        mixed.handle_voice_event(note_on_event(2, VoiceRole::Harmony, 440.0));
        let mut mixed_audio = [0.0; 1024];
        mixed.process(&mut mixed_audio, 2);

        let mut harmony = Engine::new();
        harmony.prepare(48_000, 256);
        harmony.handle_voice_event(note_on_event(2, VoiceRole::Harmony, 440.0));
        let mut harmony_audio = [0.0; 1024];
        harmony.process(&mut harmony_audio, 2);

        assert_eq!(mixed_audio, harmony_audio);
    }

    #[test]
    fn canonical_note_off_releases_only_its_voice_and_is_idempotent() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.handle_voice_event(note_on_event(10, VoiceRole::Canon, 440.0));
        e.handle_voice_event(note_on_event(11, VoiceRole::Counterpoint, 440.0));

        e.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(10),
        });
        e.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(10),
        });
        e.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(999),
        });

        assert!(!e.voices.iter().any(|v| v.owns_voice_id(VoiceId::new(10))));
        assert!(e.voices.iter().any(|v| v.owns_voice_id(VoiceId::new(11))));
    }

    #[test]
    fn canonical_panic_fast_releases_every_voice_and_pedal() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.set_sustain_pedal(true);
        e.handle_voice_event(note_on_event(1, VoiceRole::Input, 261.625_5));
        e.handle_voice_event(note_on_event(2, VoiceRole::Harmony, 329.627_56));

        e.handle_voice_event(VoiceEvent::Panic);
        assert_eq!(e.live_voice_count(), 0);
        assert!(!e.sustain_pedal());
        let mut tail = [0.0; 800];
        e.process(&mut tail, 2);
        assert_eq!(e.active_voice_count(), 0);
    }

    #[test]
    fn midi_a4_compatibility_wrapper_is_exactly_440_hz() {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        e.note_on(69, 100);
        let voice = e
            .voices
            .iter()
            .find(|v| v.owns_voice_id(VoiceId::from_midi_note(69)))
            .unwrap();
        assert_eq!(voice.frequency_hz(), 440.0);
    }

    // ─── A5 FX chain tests ──────────────────────────────────────────

    #[test]
    fn disabled_fx_slot_is_a_true_bypass() {
        use crate::fx::{Drive, FxSlot};

        let mut disabled = Engine::new();
        disabled.prepare(48_000, 256);
        disabled.set_fx_slot(0, FxSlot::Drive(Drive::with_drive(20.0)));
        disabled.set_fx_enabled(0, false);
        disabled.note_on(69, 100);

        let mut clean = Engine::new();
        clean.prepare(48_000, 256);
        clean.note_on(69, 100);

        let mut disabled_audio = [0.0; 1024];
        let mut clean_audio = [0.0; 1024];
        disabled.process(&mut disabled_audio, 2);
        clean.process(&mut clean_audio, 2);
        assert_eq!(disabled_audio, clean_audio);
    }

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

    #[test]
    fn a6_fx_variants_are_engine_reachable() {
        use crate::fx::{Chorus, Compressor, FdnReverb, Flanger, Phaser};
        assert_eq!(FX_SLOTS, 8);
        let clean = render_note_with(|_| {});
        let variants = [
            FxSlot::FdnReverb(FdnReverb::new(48_000.0)),
            FxSlot::Chorus(Chorus::new(48_000.0)),
            FxSlot::Flanger(Flanger::new(48_000.0)),
            FxSlot::Phaser(Phaser::new(48_000.0)),
            FxSlot::Compressor(Compressor::new(48_000.0)),
        ];
        for (idx, slot) in variants.into_iter().enumerate() {
            let rendered = render_note_with(|e| {
                e.set_fx_slot(idx, slot);
            });
            assert!(
                rms_diff(&clean, &rendered) > 1.0e-5,
                "{} did not affect engine render",
                rendered.len()
            );
        }
    }

    // ─── A6 oscillator integration tests ────────────────────────────

    fn render_note_with(configure: impl FnOnce(&mut Engine)) -> Vec<f32> {
        let mut e = Engine::new();
        e.prepare(48_000, 256);
        configure(&mut e);
        e.note_on(60, 100);
        let mut buf = vec![0.0f32; 4096];
        e.process(&mut buf, 2);
        buf
    }

    #[test]
    fn engine_spectral_morph_control_changes_audio() {
        let clean = render_note_with(|_| {});
        let morphed = render_note_with(|e| {
            e.set_spectral_morph(SpectralMorph::Skew);
            e.set_morph_amount(1.0);
        });
        assert!(rms_diff(&clean, &morphed) > 1.0e-3);
    }

    #[test]
    fn engine_phase_distortion_control_changes_audio() {
        let clean = render_note_with(|_| {});
        let bent = render_note_with(|e| {
            e.set_phase_distortion(PhaseDistortionMode::Sync);
            e.set_phase_amount(0.8);
        });
        assert!(rms_diff(&clean, &bent) > 1.0e-3);
    }

    #[test]
    fn engine_unison_control_changes_audio_and_stays_bounded() {
        let clean = render_note_with(|_| {});
        let unison = render_note_with(|e| {
            e.set_unison_voices(8);
            e.set_unison_style(UnisonStyle::Wide);
            e.set_unison_detune_cents(18.0);
        });
        let peak = unison.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(rms_diff(&clean, &unison) > 1.0e-4);
        assert!(peak.is_finite() && peak <= 1.0, "bad unison peak {peak}");
    }

    fn rms_diff(a: &[f32], b: &[f32]) -> f32 {
        (a.iter()
            .zip(b)
            .map(|(a, b)| {
                let d = a - b;
                d * d
            })
            .sum::<f32>()
            / a.len().min(b.len()) as f32)
            .sqrt()
    }

    // ─── A6 filter model integration tests ─────────────────────────

    #[test]
    fn engine_filter_kind_switch_keeps_audio_alive() {
        for kind in FilterKind::ALL {
            let mut e = Engine::new();
            e.prepare(48_000, 256);
            e.set_filter_kind(kind);
            e.set_filter_cutoff_hz(1_200.0);
            e.set_filter_resonance(0.55);
            e.set_filter_drive(2.0);
            e.note_on(60, 100);
            let mut buf = vec![0.0f32; 4096];
            e.process(&mut buf, 2);
            let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            assert!(peak.is_finite());
            assert!(peak > 0.0, "filter kind {kind:?} rendered silence");
        }
    }

    #[test]
    fn engine_diode_filter_changes_render_vs_svf() {
        let clean = render_note_with(|e| {
            e.set_filter_kind(FilterKind::DigitalSvf);
            e.set_filter_cutoff_hz(900.0);
            e.set_filter_resonance(0.4);
        });
        let diode = render_note_with(|e| {
            e.set_filter_kind(FilterKind::Diode);
            e.set_filter_cutoff_hz(900.0);
            e.set_filter_resonance(0.7);
            e.set_filter_drive(4.0);
        });
        assert!(rms_diff(&clean, &diode) > 1.0e-4);
    }

    #[test]
    fn engine_formant_and_phaser_filters_change_render_vs_svf() {
        let clean = render_note_with(|e| {
            e.set_filter_kind(FilterKind::DigitalSvf);
            e.set_filter_cutoff_hz(2_000.0);
        });
        let formant = render_note_with(|e| {
            e.set_filter_kind(FilterKind::Formant);
            e.set_filter_cutoff_hz(1_000.0);
            e.set_filter_morph(1.0, 0.0);
        });
        let phaser = render_note_with(|e| {
            e.set_filter_kind(FilterKind::Phaser);
            e.set_filter_cutoff_hz(1_500.0);
            e.set_filter_resonance(0.8);
        });
        assert!(rms_diff(&clean, &formant) > 1.0e-4);
        assert!(rms_diff(&clean, &phaser) > 1.0e-4);
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
