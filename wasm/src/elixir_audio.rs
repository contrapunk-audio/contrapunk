use contrapunk::slide::{
    SlideCurve, SlideRole, SlideRuntime, SlideSettings, SlideSlot, SlideTravel, SlideTrigger,
    MAX_SLIDE_VOICES,
};
use elixir_core::{Engine, VoiceEvent, VoiceId, VoiceRole};
use wasm_bindgen::prelude::*;

/// Preallocated browser-audio wrapper for use inside an AudioWorklet.
#[wasm_bindgen]
pub struct ElixirAudio {
    engine: Engine,
    output: Vec<f32>,
    max_frames: usize,
    sample_rate: u32,
    slide: SlideRuntime,
    slide_voice_ids: [u32; MAX_SLIDE_VOICES],
    slide_slots: [u8; MAX_SLIDE_VOICES],
    slide_frequencies: [f32; MAX_SLIDE_VOICES],
    slide_targets: [f32; MAX_SLIDE_VOICES],
    slide_progresses: [f32; MAX_SLIDE_VOICES],
    slide_durations: [f32; MAX_SLIDE_VOICES],
    slide_curves: [u8; MAX_SLIDE_VOICES],
    slide_count: usize,
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
            sample_rate,
            slide: SlideRuntime::new(),
            slide_voice_ids: [0; MAX_SLIDE_VOICES],
            slide_slots: [0; MAX_SLIDE_VOICES],
            slide_frequencies: [0.0; MAX_SLIDE_VOICES],
            slide_targets: [0.0; MAX_SLIDE_VOICES],
            slide_progresses: [0.0; MAX_SLIDE_VOICES],
            slide_durations: [0.0; MAX_SLIDE_VOICES],
            slide_curves: [0; MAX_SLIDE_VOICES],
            slide_count: 0,
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
        self.note_on_slide(
            voice_id,
            role,
            midi_anchor,
            frequency_hz,
            velocity,
            0,
            0,
            0.0,
            0,
            0,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn note_on_slide(
        &mut self,
        voice_id: u32,
        role: u8,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        slide_voice: u8,
        travel_kind: u8,
        travel_value: f32,
        trigger: u8,
        curve: u8,
    ) {
        let Some(role) = role_from_u8(role) else {
            return;
        };
        let slide_role = match role {
            VoiceRole::Input => SlideRole::Input,
            VoiceRole::Harmony => SlideRole::Harmony,
            VoiceRole::Canon => SlideRole::Canon,
            VoiceRole::Counterpoint => SlideRole::Counterpoint,
        };
        let settings = SlideSettings {
            travel: match travel_kind {
                1 => SlideTravel::Time {
                    milliseconds: travel_value,
                },
                2 => SlideTravel::Rate {
                    semitones_per_second: travel_value,
                },
                _ => SlideTravel::Off,
            },
            trigger: if trigger == 1 {
                SlideTrigger::Always
            } else {
                SlideTrigger::Legato
            },
            curve: match curve {
                1 => SlideCurve::Exponential,
                2 => SlideCurve::InverseExponential,
                _ => SlideCurve::Linear,
            },
        };
        let frequency_hz = self.slide.note_on(
            voice_id as u64,
            SlideSlot::new(slide_role, slide_voice),
            frequency_hz,
            settings,
            self.sample_rate as f32,
        );
        self.engine.handle_voice_event(VoiceEvent::NoteOn {
            voice_id: VoiceId::new(voice_id as u64),
            role,
            midi_anchor,
            frequency_hz,
            velocity,
        });
    }

    pub fn retune(&mut self, voice_id: u32, frequency_hz: f32) {
        self.slide.retune_now(voice_id as u64, frequency_hz);
        self.engine.handle_voice_event(VoiceEvent::Retune {
            voice_id: VoiceId::new(voice_id as u64),
            frequency_hz,
        });
    }

    pub fn note_off(&mut self, voice_id: u32) {
        self.slide.note_off(voice_id as u64);
        self.engine.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(voice_id as u64),
        });
    }

    pub fn set_sustain(&mut self, enabled: bool) {
        self.engine.set_sustain_pedal(enabled);
    }

    pub fn panic(&mut self) {
        self.slide.clear();
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

    pub fn set_role_parameter(&mut self, role: u8, parameter: u8, value: f32) {
        let Some(role) = role_from_u8(role) else {
            return;
        };
        let mut patch = self.engine.role_patch(role);
        if patch.set_parameter(parameter, value) {
            self.engine.set_role_patch(role, patch);
        }
    }

    pub fn set_pitch_bend_cents(&mut self, cents: f32) {
        self.engine.set_pitch_bend_cents(cents);
    }

    pub fn set_expression(&mut self, value: f32) {
        self.engine.set_expression(value);
    }

    pub fn set_mod_wheel(&mut self, value: f32) {
        self.engine.set_mod_wheel(value);
    }

    /// Render at most the preallocated frame bound and return the frame count.
    pub fn process(&mut self, frames: usize, channels: usize) -> usize {
        const SLIDE_UPDATE_FRAMES: usize = 8;
        let channels = channels.clamp(1, 2);
        let frames = frames.min(self.max_frames);
        let output = &mut self.output[..frames * channels];
        if self.slide.is_moving() {
            for chunk in output.chunks_mut(channels * SLIDE_UPDATE_FRAMES) {
                let engine = &mut self.engine;
                self.slide.for_each_moving(|voice_id, frequency_hz| {
                    engine.handle_voice_event(VoiceEvent::Retune {
                        voice_id: VoiceId::new(voice_id),
                        frequency_hz,
                    });
                });
                self.slide.finish_completed();
                self.engine.process(chunk, channels);
                self.slide.advance(chunk.len() / channels);
            }
        } else {
            self.engine.process(output, channels);
        }
        self.slide_count = 0;
        self.slide.for_each_moving_snapshot(|snapshot| {
            if self.slide_count < MAX_SLIDE_VOICES {
                self.slide_voice_ids[self.slide_count] = snapshot.voice_id as u32;
                self.slide_slots[self.slide_count] =
                    snapshot.slot.role as u8 | (snapshot.slot.voice << 2);
                self.slide_frequencies[self.slide_count] = snapshot.current_frequency_hz;
                self.slide_targets[self.slide_count] = snapshot.target_frequency_hz;
                self.slide_progresses[self.slide_count] = snapshot.progress;
                self.slide_durations[self.slide_count] = snapshot.duration_ms;
                self.slide_curves[self.slide_count] = snapshot.curve as u8;
                self.slide_count += 1;
            }
        });
        frames
    }

    pub fn slide_snapshot_count(&self) -> usize {
        self.slide_count
    }

    pub fn slide_voice_ids_ptr(&self) -> *const u32 {
        self.slide_voice_ids.as_ptr()
    }

    pub fn slide_slots_ptr(&self) -> *const u8 {
        self.slide_slots.as_ptr()
    }

    pub fn slide_frequencies_ptr(&self) -> *const f32 {
        self.slide_frequencies.as_ptr()
    }

    pub fn slide_targets_ptr(&self) -> *const f32 {
        self.slide_targets.as_ptr()
    }

    pub fn slide_progresses_ptr(&self) -> *const f32 {
        self.slide_progresses.as_ptr()
    }

    pub fn slide_durations_ptr(&self) -> *const f32 {
        self.slide_durations.as_ptr()
    }

    pub fn slide_curves_ptr(&self) -> *const u8 {
        self.slide_curves.as_ptr()
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
    fn slide_wrapper_reports_exact_motion_without_core_changes() {
        let mut audio = ElixirAudio::new(48_000, 128);
        audio.note_on_slide(1, 0, 69, 440.0, 100, 0, 1, 800.0, 1, 2);
        audio.process(128, 2);
        audio.note_off(1);
        audio.note_on_slide(2, 0, 76, 659.255_1, 100, 0, 1, 800.0, 1, 2);
        audio.process(128, 2);
        assert_eq!(audio.slide_snapshot_count(), 1);
        assert_eq!(audio.slide_voice_ids[0], 2);
        assert_eq!(audio.slide_slots[0], 0);
        assert!(audio.slide_frequencies[0] > 440.0);
        assert!(audio.slide_frequencies[0] < 659.255_1);
        assert!((audio.slide_targets[0] - 659.255_1).abs() < 0.001);
        assert_eq!(audio.slide_durations[0], 800.0);
        assert_eq!(audio.slide_curves[0], SlideCurve::InverseExponential as u8);
    }

    #[test]
    fn invalid_role_is_ignored() {
        let mut audio = ElixirAudio::new(48_000, 128);
        audio.note_on(1, 99, 69, 440.0, 127);
        audio.process(128, 2);
        assert!(audio.output.iter().all(|sample| *sample == 0.0));
    }
}
