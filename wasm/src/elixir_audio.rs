use elixir_core::{Engine, VoiceEvent, VoiceId, VoiceRole};
use wasm_bindgen::prelude::*;

/// Preallocated browser-audio wrapper for use inside an AudioWorklet.
#[wasm_bindgen]
pub struct ElixirAudio {
    engine: Engine,
    output: Vec<f32>,
    max_frames: usize,
}

#[wasm_bindgen]
impl ElixirAudio {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: u32, max_frames: usize) -> Self {
        let max_frames = max_frames.max(1);
        let mut engine = Engine::new();
        engine.prepare(sample_rate, max_frames);
        Self {
            engine,
            output: vec![0.0; max_frames * 2],
            max_frames,
        }
    }

    pub fn note_on(
        &mut self,
        voice_id: u32,
        role: u8,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
    ) {
        let Some(role) = role_from_u8(role) else {
            return;
        };
        self.engine.handle_voice_event(VoiceEvent::NoteOn {
            voice_id: VoiceId::new(voice_id as u64),
            role,
            midi_anchor,
            frequency_hz,
            velocity,
        });
    }

    pub fn retune(&mut self, voice_id: u32, frequency_hz: f32) {
        self.engine.handle_voice_event(VoiceEvent::Retune {
            voice_id: VoiceId::new(voice_id as u64),
            frequency_hz,
        });
    }

    pub fn note_off(&mut self, voice_id: u32) {
        self.engine.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(voice_id as u64),
        });
    }

    pub fn set_sustain(&mut self, enabled: bool) {
        self.engine.set_sustain_pedal(enabled);
    }

    pub fn panic(&mut self) {
        self.engine.handle_voice_event(VoiceEvent::Panic);
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.engine.set_master_gain(gain);
    }

    pub fn set_role_gain(&mut self, role: u8, gain: f32) {
        if let Some(role) = role_from_u8(role) {
            self.engine.set_role_gain(role, gain);
        }
    }

    /// Render at most the preallocated frame bound and return the frame count.
    pub fn process(&mut self, frames: usize, channels: usize) -> usize {
        let channels = channels.clamp(1, 2);
        let frames = frames.min(self.max_frames);
        self.engine
            .process(&mut self.output[..frames * channels], channels);
        frames
    }

    pub fn output_ptr(&self) -> *const f32 {
        self.output.as_ptr()
    }

    pub fn output_capacity(&self) -> usize {
        self.output.len()
    }
}

fn role_from_u8(role: u8) -> Option<VoiceRole> {
    VoiceRole::ALL.get(role as usize).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_is_bounded_finite_and_role_aware() {
        let mut audio = ElixirAudio::new(48_000, 128);
        audio.set_master_gain(1.0);
        audio.set_role_gain(VoiceRole::Input as u8, 0.0);
        audio.note_on(1, VoiceRole::Input as u8, 69, 440.0, 127);
        audio.note_on(2, VoiceRole::Harmony as u8, 69, 442.0, 127);
        assert_eq!(audio.process(256, 2), 128);
        assert!(audio.output.iter().all(|sample| sample.is_finite()));
        assert!(audio.output.iter().any(|sample| sample.abs() > 1.0e-6));

        audio.retune(2, 432.0);
        audio.note_off(2);
        audio.process(128, 2);
        audio.process(128, 2);
        assert!(audio.output[240..256].iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn invalid_role_is_ignored() {
        let mut audio = ElixirAudio::new(48_000, 128);
        audio.note_on(1, 99, 69, 440.0, 127);
        audio.process(128, 2);
        assert!(audio.output.iter().all(|sample| *sample == 0.0));
    }
}
