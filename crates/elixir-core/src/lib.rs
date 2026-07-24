//! Host-neutral, allocation-free Elixir sine synthesizer.
//!
//! Every voice is one fixed sine oscillator. The only sound controls are
//! master gain and Contrapunk role gains; note edges use a fixed 5 ms ramp.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

#[cfg(test)]
#[global_allocator]
static TEST_ALLOCATOR: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;

mod osc;
pub mod util;
mod voice;

use voice::Voice;

/// Stable caller-owned identity for one sounding voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VoiceId(u64);

impl VoiceId {
    pub const INVALID: Self = Self(u64::MAX);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Contrapunk routing role retained by a voice for role-level gain.
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
    Retune {
        voice_id: VoiceId,
        frequency_hz: f32,
    },
    NoteOff {
        voice_id: VoiceId,
    },
    Panic,
}

pub const MAX_POLYPHONY: usize = 16;
const VOICE_SLOTS: usize = MAX_POLYPHONY * 2;
/// Fixed, non-user-facing ramp applied to note starts, releases, steals, and panic.
pub const DECLICK_SECS: f32 = 0.005;

/// One audio-thread-owned Elixir engine.
pub struct Engine {
    sample_rate: u32,
    max_block: usize,
    voices: [Voice; VOICE_SLOTS],
    master_gain: f32,
    role_gains: [f32; VoiceRole::ALL.len()],
    sustain_pedal: bool,
    note_counter: u64,
}

impl Engine {
    pub const fn new() -> Self {
        Self {
            sample_rate: 0,
            max_block: 0,
            voices: [const { Voice::new() }; VOICE_SLOTS],
            master_gain: 0.25,
            role_gains: [1.0; VoiceRole::ALL.len()],
            sustain_pedal: false,
            note_counter: 0,
        }
    }

    /// Prepare outside the audio callback. Re-preparing panics current voices.
    pub fn prepare(&mut self, sample_rate: u32, max_block: usize) {
        self.panic();
        self.sample_rate = sample_rate;
        self.max_block = max_block;
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        if gain.is_finite() {
            self.master_gain = gain.clamp(0.0, 1.0);
        }
    }

    pub fn master_gain(&self) -> f32 {
        self.master_gain
    }

    pub fn set_role_gain(&mut self, role: VoiceRole, gain: f32) {
        if gain.is_finite() {
            self.role_gains[role.index()] = gain.clamp(0.0, 1.0);
        }
    }

    pub fn role_gain(&self, role: VoiceRole) -> f32 {
        self.role_gains[role.index()]
    }

    pub fn role_gains(&self) -> [f32; VoiceRole::ALL.len()] {
        self.role_gains
    }

    pub fn set_sustain_pedal(&mut self, on: bool) {
        let was_on = self.sustain_pedal;
        self.sustain_pedal = on;
        if was_on && !on {
            for voice in &mut self.voices {
                voice.release_sustain();
            }
        }
    }

    pub fn handle_voice_event(&mut self, event: VoiceEvent) {
        match event {
            VoiceEvent::NoteOn {
                voice_id,
                role,
                midi_anchor,
                frequency_hz,
                velocity,
            } => self.start_voice(voice_id, role, midi_anchor, frequency_hz, velocity),
            VoiceEvent::Retune {
                voice_id,
                frequency_hz,
            } => {
                for voice in &mut self.voices {
                    voice.retune(voice_id, frequency_hz, self.sample_rate as f32);
                }
            }
            VoiceEvent::NoteOff { voice_id } => {
                for voice in &mut self.voices {
                    voice.note_off(voice_id, self.sustain_pedal);
                }
            }
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
            || velocity == 0
            || !frequency_hz.is_finite()
            || frequency_hz <= 0.0
        {
            return;
        }

        // Reusing an identity crossfades rather than discontinuously resetting it.
        for voice in &mut self.voices {
            if voice.has_voice_id(voice_id) {
                voice.release();
            }
        }

        if self.live_voice_count() >= MAX_POLYPHONY {
            if let Some(oldest) = self
                .voices
                .iter_mut()
                .filter(|voice| voice.is_live())
                .min_by_key(|voice| voice.age())
            {
                oldest.release();
            }
        }

        let slot_index = self
            .voices
            .iter()
            .position(|voice| !voice.is_active())
            .or_else(|| {
                self.voices
                    .iter()
                    .enumerate()
                    .filter(|(_, voice)| !voice.is_live())
                    .min_by_key(|(_, voice)| voice.age())
                    .map(|(index, _)| index)
            });
        let Some(slot_index) = slot_index else {
            return;
        };

        let age = self.note_counter;
        self.note_counter = self.note_counter.wrapping_add(1);
        self.voices[slot_index].start(
            voice_id,
            role,
            midi_anchor,
            frequency_hz,
            velocity,
            self.sample_rate as f32,
            age,
        );
    }

    /// Release every owned voice through the fixed de-click ramp.
    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.release();
        }
        self.sustain_pedal = false;
    }

    /// Panic and teardown use the same short bounded ramp; no voice remains live.
    pub fn panic(&mut self) {
        self.all_notes_off();
    }

    /// Overwrite an interleaved output buffer without allocation or blocking.
    pub fn process(&mut self, buffer: &mut [f32], channels: usize) {
        buffer.fill(0.0);
        if channels == 0 || self.sample_rate == 0 {
            return;
        }

        let role_gains = self.role_gains;
        for frame in buffer.chunks_exact_mut(channels) {
            let mut mix = 0.0;
            for voice in &mut self.voices {
                mix += voice.tick() * role_gains[voice.role().index()];
            }
            let sample = mix * self.master_gain;
            let sample = if sample.is_finite() { sample } else { 0.0 };
            for channel in frame {
                *channel = sample;
            }
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn max_block(&self) -> usize {
        self.max_block
    }

    pub fn live_voice_count(&self) -> usize {
        self.voices.iter().filter(|voice| voice.is_live()).count()
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|voice| voice.is_active()).count()
    }

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

    fn note_on(id: u64, role: VoiceRole, anchor: u8, frequency_hz: f32) -> VoiceEvent {
        VoiceEvent::NoteOn {
            voice_id: VoiceId::new(id),
            role,
            midi_anchor: anchor,
            frequency_hz,
            velocity: 100,
        }
    }

    fn render(engine: &mut Engine, frames: usize) -> [f32; 512] {
        assert!(frames <= 256);
        let mut output = [0.0; 512];
        engine.process(&mut output[..frames * 2], 2);
        output
    }

    #[test]
    fn golden_a4_starts_as_exact_sine_with_fixed_declick_ramp() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        engine.set_master_gain(1.0);
        engine.handle_voice_event(VoiceEvent::NoteOn {
            voice_id: VoiceId::new(1),
            role: VoiceRole::Input,
            midi_anchor: 69,
            frequency_hz: 440.0,
            velocity: 127,
        });
        let mut output = [0.0; 2];
        engine.process(&mut output, 1);
        let ramp_frames = DECLICK_SECS * 48_000.0;
        let expected = libm::sinf(core::f32::consts::TAU * 440.0 / 48_000.0) * (2.0 / ramp_frames);
        assert_eq!(output[0], 0.0);
        assert!((output[1] - expected).abs() < 1.0e-7);
    }

    #[test]
    fn canonical_metadata_and_exact_frequency_are_retained() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        engine.handle_voice_event(note_on(7, VoiceRole::Canon, 69, 442.125));
        assert!(engine
            .voices
            .iter()
            .any(|voice| { voice.metadata() == (VoiceId::new(7), VoiceRole::Canon, 69, 442.125) }));
    }

    #[test]
    fn sixteen_voices_play_and_seventeenth_steals_oldest() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        for id in 0..MAX_POLYPHONY as u64 {
            engine.handle_voice_event(note_on(id, VoiceRole::Harmony, 60, 220.0 + id as f32));
        }
        assert_eq!(engine.live_voice_count(), MAX_POLYPHONY);
        engine.handle_voice_event(note_on(99, VoiceRole::Harmony, 72, 523.251_1));
        assert_eq!(engine.live_voice_count(), MAX_POLYPHONY);
        assert!(engine
            .voices
            .iter()
            .any(|voice| { voice.metadata().0 == VoiceId::new(99) && voice.is_live() }));
        assert!(!engine
            .voices
            .iter()
            .any(|voice| { voice.metadata().0 == VoiceId::new(0) && voice.is_live() }));
    }

    #[test]
    fn retune_preserves_identity_phase_and_ownership() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        engine.handle_voice_event(note_on(1, VoiceRole::Input, 69, 440.0));
        let mut before = [0.0; 64];
        engine.process(&mut before, 1);
        engine.handle_voice_event(VoiceEvent::Retune {
            voice_id: VoiceId::new(1),
            frequency_hz: 432.0,
        });
        assert_eq!(engine.live_voice_count(), 1);
        assert!(engine
            .voices
            .iter()
            .any(|voice| { voice.metadata() == (VoiceId::new(1), VoiceRole::Input, 69, 432.0) }));
        let mut after = [0.0; 2];
        engine.process(&mut after, 1);
        assert_ne!(after[0], 0.0, "retune must not reset oscillator phase");
        engine.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(1),
        });
        assert_eq!(engine.live_voice_count(), 0);
    }

    #[test]
    fn note_off_is_identity_exact_and_idempotent() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        engine.handle_voice_event(note_on(1, VoiceRole::Input, 69, 440.0));
        engine.handle_voice_event(note_on(2, VoiceRole::Input, 69, 442.0));
        engine.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(1),
        });
        engine.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(1),
        });
        assert_eq!(engine.live_voice_count(), 1);
        assert!(engine
            .voices
            .iter()
            .any(|voice| { voice.metadata().0 == VoiceId::new(2) && voice.is_live() }));
    }

    #[test]
    fn sustain_owns_release_until_pedal_up() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        engine.set_sustain_pedal(true);
        engine.handle_voice_event(note_on(1, VoiceRole::Input, 60, 261.625_55));
        engine.handle_voice_event(VoiceEvent::NoteOff {
            voice_id: VoiceId::new(1),
        });
        let held = render(&mut engine, 256);
        assert!(held.iter().any(|sample| sample.abs() > 1.0e-6));
        engine.set_sustain_pedal(false);
        let _ = render(&mut engine, 256);
        assert_eq!(engine.active_voice_count(), 0);
    }

    #[test]
    fn panic_is_bounded_and_clears_pedal_and_live_ownership() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        engine.set_sustain_pedal(true);
        engine.handle_voice_event(note_on(1, VoiceRole::Input, 69, 440.0));
        let _ = render(&mut engine, 256);
        engine.handle_voice_event(VoiceEvent::Panic);
        assert_eq!(engine.live_voice_count(), 0);
        assert!(!engine.sustain_pedal());
        let tail = render(&mut engine, 256);
        assert!(tail.iter().all(|sample| sample.is_finite()));
        assert_eq!(engine.active_voice_count(), 0);
    }

    #[test]
    fn role_and_master_gains_are_the_only_sound_controls() {
        let mut muted = Engine::new();
        muted.prepare(48_000, 256);
        muted.set_role_gain(VoiceRole::Counterpoint, 0.0);
        muted.handle_voice_event(note_on(1, VoiceRole::Counterpoint, 69, 440.0));
        assert!(render(&mut muted, 256).iter().all(|sample| *sample == 0.0));

        muted.set_master_gain(f32::NAN);
        assert_eq!(muted.master_gain(), 0.25);
    }

    #[test]
    fn process_is_finite_and_allocation_free() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        let mut output = [0.0; 512];
        assert_no_alloc::assert_no_alloc(|| {
            engine.handle_voice_event(note_on(1, VoiceRole::Input, 69, 440.0));
            engine.process(&mut output, 2);
            engine.handle_voice_event(VoiceEvent::NoteOff {
                voice_id: VoiceId::new(1),
            });
            engine.process(&mut output, 2);
            engine.handle_voice_event(VoiceEvent::Panic);
            engine.process(&mut output, 2);
        });
        assert!(output.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn invalid_events_are_safe_noops() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        for event in [
            note_on(1, VoiceRole::Input, 128, 440.0),
            note_on(2, VoiceRole::Input, 69, f32::NAN),
            VoiceEvent::NoteOn {
                voice_id: VoiceId::INVALID,
                role: VoiceRole::Input,
                midi_anchor: 69,
                frequency_hz: 440.0,
                velocity: 100,
            },
        ] {
            engine.handle_voice_event(event);
        }
        assert_eq!(engine.active_voice_count(), 0);
    }
}
