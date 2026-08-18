//! One role-aware Elixir voice with harmonic colour and independent articulation.

use crate::env::Envelope;
use crate::osc::Oscillator;
use crate::{RolePatch, VoiceId, VoiceRole};

pub(crate) struct Voice {
    oscillator: Oscillator,
    envelope: Envelope,
    patch: RolePatch,
    patch_smoothing: f32,
    active: bool,
    released: bool,
    sustained: bool,
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
            oscillator: Oscillator::new(),
            envelope: Envelope::new(),
            patch: RolePatch::sine(),
            patch_smoothing: 1.0,
            active: false,
            released: false,
            sustained: false,
            voice_id: VoiceId::INVALID,
            role: VoiceRole::Input,
            midi_anchor: 0,
            frequency_hz: 0.0,
            velocity: 0.0,
            age: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        voice_id: VoiceId,
        role: VoiceRole,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        sample_rate: f32,
        age: u64,
        patch: RolePatch,
    ) {
        self.patch = patch;
        self.patch_smoothing = 1.0 - libm::expf(-1.0 / (0.01 * sample_rate.max(1.0)));
        self.oscillator.start(patch);
        self.envelope.note_on();
        self.active = true;
        self.released = false;
        self.sustained = false;
        self.voice_id = voice_id;
        self.role = role;
        self.midi_anchor = midi_anchor;
        self.frequency_hz = frequency_hz;
        self.velocity = velocity as f32 / 127.0;
        self.age = age;
    }

    pub fn retune(&mut self, voice_id: VoiceId, frequency_hz: f32) {
        if self.has_voice_id(voice_id) && frequency_hz.is_finite() && frequency_hz > 0.0 {
            self.frequency_hz = frequency_hz;
        }
    }

    pub fn note_off(&mut self, voice_id: VoiceId, sustain_down: bool) {
        if !self.is_live() || self.voice_id != voice_id {
            return;
        }
        self.released = true;
        if sustain_down {
            self.sustained = true;
        } else {
            self.begin_release();
        }
    }

    pub fn release_sustain(&mut self) {
        if self.sustained {
            self.sustained = false;
            self.begin_release();
        }
    }

    pub fn release(&mut self) {
        if self.active && (!self.released || self.sustained) {
            self.released = true;
            self.sustained = false;
            self.begin_release();
        }
    }

    fn begin_release(&mut self) {
        self.envelope.note_off();
    }

    #[inline]
    pub fn tick(
        &mut self,
        target_patch: RolePatch,
        pitch_bend_cents: f32,
        expression: f32,
        mod_wheel: f32,
        sample_rate: f32,
    ) -> f32 {
        if !self.active {
            return 0.0;
        }

        self.patch.smooth_toward(target_patch, self.patch_smoothing);
        let envelope = self.envelope.tick(self.patch.envelope, sample_rate);
        if self.released && self.envelope.is_idle() {
            self.deactivate();
            return 0.0;
        }

        let velocity_gain = 1.0 + (self.velocity - 1.0) * self.patch.envelope.velocity_sensitivity;
        let expression_gain = 1.0 + (expression - 1.0) * self.patch.envelope.expression_sensitivity;
        self.oscillator.tick(
            self.frequency_hz,
            self.patch,
            pitch_bend_cents,
            mod_wheel,
            sample_rate,
            self.patch_smoothing,
        ) * envelope
            * velocity_gain
            * expression_gain
    }

    fn deactivate(&mut self) {
        self.active = false;
        self.released = false;
        self.sustained = false;
        self.voice_id = VoiceId::INVALID;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_live(&self) -> bool {
        self.active && !self.released
    }

    pub fn has_voice_id(&self, voice_id: VoiceId) -> bool {
        self.active && self.voice_id == voice_id
    }

    pub fn role(&self) -> VoiceRole {
        self.role
    }

    pub fn age(&self) -> u64 {
        self.age
    }

    #[cfg(test)]
    pub fn metadata(&self) -> (VoiceId, VoiceRole, u8, f32) {
        (
            self.voice_id,
            self.role,
            self.midi_anchor,
            self.frequency_hz,
        )
    }

    #[cfg(test)]
    pub fn oscillator_phase(&self) -> f32 {
        self.oscillator.primary_phase()
    }
}
