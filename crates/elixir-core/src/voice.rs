//! One fixed-sine Elixir voice with a non-user-facing de-click ramp.

use crate::osc::Oscillator;
use crate::{VoiceId, VoiceRole, DECLICK_SECS};

pub(crate) struct Voice {
    oscillator: Oscillator,
    active: bool,
    released: bool,
    sustained: bool,
    voice_id: VoiceId,
    role: VoiceRole,
    midi_anchor: u8,
    frequency_hz: f32,
    velocity: f32,
    gain: f32,
    ramp_step: f32,
    ramp_frames: f32,
    age: u64,
}

impl Voice {
    pub const fn new() -> Self {
        Self {
            oscillator: Oscillator::new(),
            active: false,
            released: false,
            sustained: false,
            voice_id: VoiceId::INVALID,
            role: VoiceRole::Input,
            midi_anchor: 0,
            frequency_hz: 0.0,
            velocity: 0.0,
            gain: 0.0,
            ramp_step: 0.0,
            ramp_frames: 1.0,
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
    ) {
        self.oscillator.start(frequency_hz, sample_rate);
        self.active = true;
        self.released = false;
        self.sustained = false;
        self.voice_id = voice_id;
        self.role = role;
        self.midi_anchor = midi_anchor;
        self.frequency_hz = frequency_hz;
        self.velocity = velocity as f32 / 127.0;
        self.gain = 0.0;
        self.ramp_frames = (DECLICK_SECS * sample_rate).max(1.0);
        self.ramp_step = 1.0 / self.ramp_frames;
        self.age = age;
    }

    pub fn retune(&mut self, voice_id: VoiceId, frequency_hz: f32, sample_rate: f32) {
        if self.has_voice_id(voice_id) && frequency_hz.is_finite() && frequency_hz > 0.0 {
            self.oscillator.retune(frequency_hz, sample_rate);
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
        self.ramp_step = -(self.gain / self.ramp_frames).max(f32::EPSILON);
    }

    #[inline]
    pub fn tick(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        if self.ramp_step > 0.0 && self.gain < 1.0 {
            self.gain = (self.gain + self.ramp_step).min(1.0);
        } else if self.ramp_step < 0.0 {
            self.gain += self.ramp_step;
            if self.gain <= 0.0 {
                self.deactivate();
                return 0.0;
            }
        }

        self.oscillator.tick() * self.velocity * self.gain
    }

    fn deactivate(&mut self) {
        self.active = false;
        self.released = false;
        self.sustained = false;
        self.voice_id = VoiceId::INVALID;
        self.gain = 0.0;
        self.ramp_step = 0.0;
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
}
