//! Synth voice (Phases 21.A1 + 21.A2).
//!
//! A1 was a single-voice scaffold. A2 keeps the same `Voice` struct
//! shape but adds the bookkeeping the engine needs to run a pool:
//!
//! - `age` — monotonic note-on counter so the handler can find the
//!   oldest live voice when stealing.
//! - `killing` — voice was stolen and is fading via [`AdsrEnvelope::kill`];
//!   it does not count toward live polyphony.
//! - `sustained` — note-off arrived while the sustain pedal was down;
//!   the voice keeps playing until the pedal releases.
//!
//! Future A2 follow-up: SIMD-packed `AggregateVoice` (two voices per
//! `f32x8` lane), per the design doc. Not required for correctness.

use crate::env::AdsrEnvelope;
use crate::filter::{FilterCoeffs, FilterKind, FilterModel, FilterParams, SvfCoeffs};
use crate::osc::{OscParams, Oscillator};
use crate::tables::SineTable;
use crate::{VoiceId, VoiceRole};

pub struct Voice {
    osc: Oscillator,
    env: AdsrEnvelope,
    filter: FilterModel,
    legacy_compatibility: bool,
    legacy_phase: f32,
    legacy_phase_inc: f32,
    legacy_lp: f32,
    active: bool,
    killing: bool,
    sustained: bool,
    released: bool,
    voice_id: VoiceId,
    role: VoiceRole,
    midi_anchor: u8,
    frequency_hz: f32,
    velocity: f32,
    age: u64,
}

impl Voice {
    pub const fn new() -> Self {
        Self {
            osc: Oscillator::new(),
            env: AdsrEnvelope::new(),
            filter: FilterModel::new(),
            legacy_compatibility: false,
            legacy_phase: 0.0,
            legacy_phase_inc: 0.0,
            legacy_lp: 0.0,
            active: false,
            killing: false,
            sustained: false,
            released: false,
            voice_id: VoiceId::INVALID,
            role: VoiceRole::Input,
            midi_anchor: 0,
            frequency_hz: 0.0,
            velocity: 0.0,
            age: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.env.set_sample_rate(sr);
    }

    pub fn set_amp_attack_secs(&mut self, s: f32) {
        self.env.set_attack(s);
    }
    pub fn set_amp_decay_secs(&mut self, s: f32) {
        self.env.set_decay(s);
    }
    pub fn set_amp_sustain(&mut self, l: f32) {
        self.env.set_sustain(l);
    }
    pub fn set_amp_release_secs(&mut self, s: f32) {
        self.env.set_release(s);
    }

    pub fn set_filter_kind(&mut self, kind: FilterKind) {
        self.filter.set_kind(kind);
    }

    pub fn set_legacy_compatibility(&mut self, enabled: bool) {
        self.legacy_compatibility = enabled;
    }

    pub fn note_on(
        &mut self,
        voice_id: VoiceId,
        role: VoiceRole,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        sample_rate: f32,
        age: u64,
    ) {
        self.active = true;
        self.killing = false;
        self.sustained = false;
        self.released = false;
        self.voice_id = voice_id;
        self.role = role;
        self.midi_anchor = midi_anchor;
        self.frequency_hz = frequency_hz;
        self.velocity = (velocity as f32 / 127.0).clamp(0.0, 1.0);
        self.osc.set_frequency(frequency_hz, sample_rate);
        self.osc.reset_phase();
        self.legacy_phase = 0.0;
        self.legacy_phase_inc = frequency_hz * core::f32::consts::TAU / sample_rate;
        self.legacy_lp = 0.0;
        self.env.set_sample_rate(sample_rate);
        if self.legacy_compatibility {
            self.env.note_on_reset();
        } else {
            self.env.note_on();
        }
        self.filter.reset();
        self.age = age;
    }

    /// Handle note-off. If `sustain_down` is true and the voice matches
    /// the requested ID, mark it sustained instead of releasing.
    pub fn note_off_or_sustain(&mut self, voice_id: VoiceId, sustain_down: bool) {
        if !self.active || self.killing || self.released || self.voice_id != voice_id {
            return;
        }
        self.released = true;
        if sustain_down {
            self.sustained = true;
        } else {
            self.env.note_off();
        }
    }

    /// Called when the sustain pedal lifts. Drops any held notes into
    /// release.
    pub fn release_sustain(&mut self) {
        if self.sustained {
            self.sustained = false;
            self.env.note_off();
        }
    }

    /// Force a fast-release ramp (see [`AdsrEnvelope::kill`]). The voice
    /// no longer counts toward live polyphony immediately, so the slot
    /// is reusable as soon as the ramp completes (or even sooner — the
    /// engine just won't keep it in the live count).
    pub fn kill(&mut self) {
        if self.active && !self.killing {
            self.killing = true;
            self.sustained = false;
            self.released = true;
            self.env.kill();
        }
    }

    pub fn all_notes_off(&mut self) {
        if self.active {
            self.sustained = false;
            self.released = true;
            self.env.note_off();
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Counts toward live polyphony: active and not currently being killed.
    pub fn is_live(&self) -> bool {
        self.active && !self.killing
    }

    /// Whether this active, non-killing slot belongs to `voice_id`.
    pub fn has_voice_id(&self, voice_id: VoiceId) -> bool {
        self.is_live() && self.voice_id == voice_id
    }

    pub fn owns_voice_id(&self, voice_id: VoiceId) -> bool {
        self.has_voice_id(voice_id) && !self.released
    }

    pub fn voice_id(&self) -> VoiceId {
        self.voice_id
    }
    pub fn role(&self) -> VoiceRole {
        self.role
    }
    pub fn midi_anchor(&self) -> u8 {
        self.midi_anchor
    }
    pub fn frequency_hz(&self) -> f32 {
        self.frequency_hz
    }
    pub fn age(&self) -> u64 {
        self.age
    }
    pub fn is_killing(&self) -> bool {
        self.killing
    }
    pub fn is_sustained(&self) -> bool {
        self.sustained
    }
    /// Read-only access to the envelope's current output. Used by the
    /// modulation matrix as the `AmpEnv` source.
    pub fn env_value(&self) -> f32 {
        self.env.value()
    }

    /// Produce one sample. Returns 0.0 and self-deactivates when the
    /// envelope hits idle. The voice signal path is
    /// `osc → SVF lowpass → env*velocity` (A4).
    #[inline]
    pub fn tick(&mut self, table: &SineTable, filter_coeffs: &SvfCoeffs) -> f32 {
        self.tick_with_osc_params(table, filter_coeffs, &OscParams::default())
    }

    /// Produce one sample using the engine-level oscillator parameters.
    #[inline]
    pub fn tick_with_osc_params(
        &mut self,
        table: &SineTable,
        filter_coeffs: &SvfCoeffs,
        osc_params: &OscParams,
    ) -> f32 {
        if !self.active {
            return 0.0;
        }
        let env_v = self.env.tick();
        if env_v <= 0.0 && self.env.is_idle() {
            self.active = false;
            self.killing = false;
            self.sustained = false;
            return 0.0;
        }
        let osc_v = self.osc.tick_with_params(table, osc_params);
        let filtered = match &mut self.filter {
            FilterModel::DigitalSvf(svf) => svf.tick_lp(osc_v, filter_coeffs),
            other => other.tick(osc_v, &FilterParams::digital_svf(20_000.0, 0.0, 48_000.0)),
        };
        filtered * env_v * self.velocity
    }

    /// Produce one sample using engine-level oscillator and filter params.
    /// **Slow path:** re-derives filter coefficients on every sample.
    /// Prefer [`Voice::tick_with_filter_coeffs`] in the audio callback.
    #[inline]
    pub fn tick_with_filter_params(
        &mut self,
        table: &SineTable,
        filter_params: &FilterParams,
        osc_params: &OscParams,
    ) -> f32 {
        if !self.active {
            return 0.0;
        }
        let env_v = self.env.tick();
        if env_v <= 0.0 && self.env.is_idle() {
            self.active = false;
            self.killing = false;
            self.sustained = false;
            return 0.0;
        }
        let osc_v = self.osc.tick_with_params(table, osc_params);
        let filtered = self.filter.tick(osc_v, filter_params);
        filtered * env_v * self.velocity
    }

    #[inline]
    pub fn tick_legacy(&mut self, one_pole_alpha: f32) -> f32 {
        if !self.active {
            return 0.0;
        }
        let env_v = self.env.tick_with_legacy_release(true);
        if env_v <= 0.0 && self.env.is_idle() {
            self.active = false;
            self.killing = false;
            self.sustained = false;
            return 0.0;
        }
        let osc_v = libm::sinf(self.legacy_phase);
        self.legacy_phase += self.legacy_phase_inc;
        if self.legacy_phase >= core::f32::consts::TAU {
            self.legacy_phase -= core::f32::consts::TAU;
        }
        let input = osc_v * env_v * self.velocity;
        self.legacy_lp += one_pole_alpha * (input - self.legacy_lp);
        self.legacy_lp
    }

    /// Produce one sample using pre-computed per-block filter
    /// coefficients. The audio-callback hot path — zero `tanf`, zero
    /// allocation.
    #[inline]
    pub fn tick_with_filter_coeffs(
        &mut self,
        table: &SineTable,
        filter_coeffs: &FilterCoeffs,
        osc_params: &OscParams,
    ) -> f32 {
        if !self.active {
            return 0.0;
        }
        let env_v = self.env.tick();
        if env_v <= 0.0 && self.env.is_idle() {
            self.active = false;
            self.killing = false;
            self.sustained = false;
            return 0.0;
        }
        let osc_v = self.osc.tick_with_params(table, osc_params);
        let filtered = self.filter.tick_prepared(osc_v, filter_coeffs);
        filtered * env_v * self.velocity
    }
}

impl Default for Voice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bypass_coeffs() -> SvfCoeffs {
        // Practical bypass — cutoff at the top of the clamp range, no
        // resonance. Audio band passes through with negligible
        // attenuation. (`SvfCoeffs::identity()` is documentary-only;
        // the SVF math collapses to v2=ic2eq with `a2=a3=0`.)
        SvfCoeffs::from_params(20_000.0, 0.0, 48_000.0)
    }

    #[test]
    fn fresh_voice_is_silent_and_inactive() {
        let table = SineTable::new();
        let bypass = bypass_coeffs();
        let mut v = Voice::new();
        assert!(!v.is_active());
        for _ in 0..128 {
            assert_eq!(v.tick(&table, &bypass), 0.0);
        }
    }

    #[test]
    fn note_on_makes_voice_active_and_audible() {
        let table = SineTable::new();
        let bypass = bypass_coeffs();
        let mut v = Voice::new();
        v.note_on(
            VoiceId::new(69),
            VoiceRole::Input,
            69,
            440.0,
            100,
            48_000.0,
            0,
        );
        assert!(v.is_active());
        let mut any_nonzero = false;
        for _ in 0..2048 {
            if v.tick(&table, &bypass).abs() > 1e-6 {
                any_nonzero = true;
            }
        }
        assert!(any_nonzero);
    }

    #[test]
    fn note_off_sends_voice_to_idle() {
        let table = SineTable::new();
        let bypass = bypass_coeffs();
        let mut v = Voice::new();
        let id = VoiceId::new(60);
        v.note_on(id, VoiceRole::Input, 60, 261.625_5, 100, 48_000.0, 0);
        for _ in 0..(48_000 / 4) {
            let _ = v.tick(&table, &bypass);
        }
        v.note_off_or_sustain(id, false);
        for _ in 0..30_000 {
            let _ = v.tick(&table, &bypass);
        }
        assert!(!v.is_active());
    }

    #[test]
    fn note_off_only_releases_matching_note() {
        let table = SineTable::new();
        let bypass = bypass_coeffs();
        let mut v = Voice::new();
        v.note_on(
            VoiceId::new(60),
            VoiceRole::Input,
            60,
            261.625_5,
            100,
            48_000.0,
            0,
        );
        v.note_off_or_sustain(VoiceId::new(72), false);
        for _ in 0..100 {
            let _ = v.tick(&table, &bypass);
        }
        assert!(v.is_active());
    }

    #[test]
    fn kill_demotes_voice_immediately_and_silences_within_5ms() {
        let table = SineTable::new();
        let bypass = bypass_coeffs();
        let mut v = Voice::new();
        v.note_on(
            VoiceId::new(60),
            VoiceRole::Input,
            60,
            261.625_5,
            127,
            48_000.0,
            0,
        );
        // settle into sustain
        for _ in 0..2048 {
            let _ = v.tick(&table, &bypass);
        }
        assert!(v.is_live());
        v.kill();
        assert!(!v.is_live());
        assert!(v.is_killing());
        // 5 ms = 240 samples at 48 kHz
        for _ in 0..400 {
            let _ = v.tick(&table, &bypass);
        }
        assert!(!v.is_active());
    }

    #[test]
    fn sustain_holds_through_note_off_and_releases_on_pedal_up() {
        let table = SineTable::new();
        let bypass = bypass_coeffs();
        let mut v = Voice::new();
        let id = VoiceId::new(60);
        v.note_on(id, VoiceRole::Input, 60, 261.625_5, 100, 48_000.0, 0);
        for _ in 0..2048 {
            let _ = v.tick(&table, &bypass);
        }
        // pedal down → note-off marks sustained, voice keeps playing
        v.note_off_or_sustain(id, true);
        assert!(v.is_sustained());
        assert!(v.is_live() || v.is_active());
        for _ in 0..2048 {
            let _ = v.tick(&table, &bypass);
        }
        assert!(v.is_active());
        // pedal up → release
        v.release_sustain();
        assert!(!v.is_sustained());
        for _ in 0..30_000 {
            let _ = v.tick(&table, &bypass);
        }
        assert!(!v.is_active());
    }
}
