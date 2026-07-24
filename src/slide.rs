//! Renderer-neutral pitch trajectories for Contrapunk's Slide feature.
//!
//! Slide is resolved on note/control events, outside renderers. The fixed-size
//! runtime only tracks stable arrangement slots and caller-owned voice IDs; a
//! renderer consumes the resulting exact frequencies through its existing
//! retune path.

use serde::{Deserialize, Serialize};

pub const SLIDE_ROLE_COUNT: usize = 4;
pub const SLIDE_VOICES_PER_ROLE: usize = 8;
pub const MAX_SLIDE_VOICES: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideRole {
    #[default]
    Input,
    Harmony,
    Canon,
    Counterpoint,
}

impl SlideRole {
    pub const ALL: [Self; SLIDE_ROLE_COUNT] =
        [Self::Input, Self::Harmony, Self::Canon, Self::Counterpoint];

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlideSlot {
    pub role: SlideRole,
    pub voice: u8,
}

impl SlideSlot {
    pub const fn new(role: SlideRole, voice: u8) -> Self {
        Self { role, voice }
    }

    const fn index(self) -> Option<usize> {
        if self.voice as usize >= SLIDE_VOICES_PER_ROLE {
            None
        } else {
            Some(self.role.index() * SLIDE_VOICES_PER_ROLE + self.voice as usize)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideTrigger {
    #[default]
    Legato,
    Always,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideCurve {
    #[default]
    Linear,
    Exponential,
    InverseExponential,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SlideTravel {
    #[default]
    Off,
    Time {
        milliseconds: f32,
    },
    Rate {
        semitones_per_second: f32,
    },
}

impl SlideTravel {
    pub fn validate(self) -> bool {
        match self {
            Self::Off => true,
            Self::Time { milliseconds } => {
                milliseconds.is_finite() && (1.0..=5_000.0).contains(&milliseconds)
            }
            Self::Rate {
                semitones_per_second,
            } => semitones_per_second.is_finite() && (0.1..=96.0).contains(&semitones_per_second),
        }
    }

    fn samples(self, start_hz: f32, target_hz: f32, sample_rate: f32) -> Option<u32> {
        let seconds = match self {
            Self::Off => return None,
            Self::Time { milliseconds } => milliseconds / 1_000.0,
            Self::Rate {
                semitones_per_second,
            } => {
                let semitones = 12.0 * (target_hz / start_hz).log2().abs();
                semitones / semitones_per_second
            }
        };
        if !seconds.is_finite() || seconds <= 0.0 {
            return None;
        }
        Some((seconds * sample_rate).round().clamp(1.0, u32::MAX as f32) as u32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlideSettings {
    pub travel: SlideTravel,
    pub trigger: SlideTrigger,
    pub curve: SlideCurve,
}

impl Default for SlideSettings {
    fn default() -> Self {
        Self {
            travel: SlideTravel::Off,
            trigger: SlideTrigger::Legato,
            curve: SlideCurve::Linear,
        }
    }
}

impl SlideSettings {
    pub fn validate(self) -> bool {
        self.travel.validate()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SlideOverride {
    pub travel: Option<SlideTravel>,
    pub trigger: Option<SlideTrigger>,
    pub curve: Option<SlideCurve>,
}

impl SlideOverride {
    pub fn resolve(self, parent: SlideSettings) -> SlideSettings {
        SlideSettings {
            travel: self.travel.unwrap_or(parent.travel),
            trigger: self.trigger.unwrap_or(parent.trigger),
            curve: self.curve.unwrap_or(parent.curve),
        }
    }

    pub fn validate(self) -> bool {
        self.travel.is_none_or(SlideTravel::validate)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlideConfig {
    pub roles: [SlideSettings; SLIDE_ROLE_COUNT],
    pub voices: [[SlideOverride; SLIDE_VOICES_PER_ROLE]; SLIDE_ROLE_COUNT],
}

impl Default for SlideConfig {
    fn default() -> Self {
        Self {
            roles: [SlideSettings::default(); SLIDE_ROLE_COUNT],
            voices: [[SlideOverride::default(); SLIDE_VOICES_PER_ROLE]; SLIDE_ROLE_COUNT],
        }
    }
}

impl SlideConfig {
    pub fn resolve(&self, slot: SlideSlot) -> Option<SlideSettings> {
        slot.index()?;
        Some(
            self.voices[slot.role.index()][slot.voice as usize]
                .resolve(self.roles[slot.role.index()]),
        )
    }

    pub fn validate(&self) -> bool {
        self.roles.iter().copied().all(SlideSettings::validate)
            && self
                .voices
                .iter()
                .flatten()
                .copied()
                .all(SlideOverride::validate)
    }
}

#[derive(Clone, Copy, Debug)]
struct SlotState {
    current_log2: f32,
    latest_voice_id: u64,
    has_history: bool,
}

impl SlotState {
    const EMPTY: Self = Self {
        current_log2: 0.0,
        latest_voice_id: u64::MAX,
        has_history: false,
    };
}

#[derive(Clone, Copy, Debug)]
struct VoiceState {
    voice_id: u64,
    slot_index: usize,
    start_log2: f32,
    target_log2: f32,
    elapsed_samples: u32,
    total_samples: u32,
    curve: SlideCurve,
    live: bool,
    moving: bool,
}

impl VoiceState {
    const EMPTY: Self = Self {
        voice_id: u64::MAX,
        slot_index: 0,
        start_log2: 0.0,
        target_log2: 0.0,
        elapsed_samples: 0,
        total_samples: 0,
        curve: SlideCurve::Linear,
        live: false,
        moving: false,
    };

    fn frequency(self) -> f32 {
        let progress = if self.total_samples == 0 {
            1.0
        } else {
            self.elapsed_samples as f32 / self.total_samples as f32
        }
        .clamp(0.0, 1.0);
        let shaped = shape(progress, self.curve);
        (self.start_log2 + (self.target_log2 - self.start_log2) * shaped).exp2()
    }
}

/// Fixed-capacity voice/slot state. Callers retain NoteOn/NoteOff ownership;
/// this runtime only supplies initial and advancing exact frequencies.
pub struct SlideRuntime {
    slots: [SlotState; MAX_SLIDE_VOICES],
    voices: [VoiceState; MAX_SLIDE_VOICES],
}

impl Default for SlideRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideRuntime {
    pub const fn new() -> Self {
        Self {
            slots: [SlotState::EMPTY; MAX_SLIDE_VOICES],
            voices: [VoiceState::EMPTY; MAX_SLIDE_VOICES],
        }
    }

    /// Register a new caller-owned voice and return its initial sounding
    /// frequency. Invalid settings or frequencies safely bypass Slide.
    pub fn note_on(
        &mut self,
        voice_id: u64,
        slot: SlideSlot,
        target_hz: f32,
        settings: SlideSettings,
        sample_rate: f32,
    ) -> f32 {
        let Some(slot_index) = slot.index() else {
            return target_hz;
        };
        if voice_id == u64::MAX
            || !target_hz.is_finite()
            || target_hz <= 0.0
            || !sample_rate.is_finite()
            || sample_rate <= 0.0
            || !settings.validate()
        {
            return target_hz;
        }

        let legato = self
            .voices
            .iter()
            .any(|voice| voice.live && voice.slot_index == slot_index);
        let slot_state = self.slots[slot_index];
        let should_slide =
            slot_state.has_history && (settings.trigger == SlideTrigger::Always || legato);
        let start_log2 = if should_slide {
            slot_state.current_log2
        } else {
            target_hz.log2()
        };
        let target_log2 = target_hz.log2();
        let total_samples = settings
            .travel
            .samples(start_log2.exp2(), target_hz, sample_rate)
            .unwrap_or(0);
        let moving =
            should_slide && total_samples > 0 && (start_log2 - target_log2).abs() > f32::EPSILON;

        let Some(index) = self.voices.iter().position(|voice| !voice.live) else {
            return target_hz;
        };
        self.voices[index] = VoiceState {
            voice_id,
            slot_index,
            start_log2,
            target_log2,
            elapsed_samples: 0,
            total_samples,
            curve: settings.curve,
            live: true,
            moving,
        };
        self.slots[slot_index] = SlotState {
            current_log2: if moving { start_log2 } else { target_log2 },
            latest_voice_id: voice_id,
            has_history: true,
        };
        (if moving { start_log2 } else { target_log2 }).exp2()
    }

    pub fn note_off(&mut self, voice_id: u64) {
        for voice in &mut self.voices {
            if voice.live && voice.voice_id == voice_id {
                voice.live = false;
                voice.moving = false;
            }
        }
    }

    /// Cancel pitch motion for one voice after an immediate external retune
    /// such as tuning Compare. Ownership remains unchanged.
    pub fn retune_now(&mut self, voice_id: u64, frequency_hz: f32) {
        if !frequency_hz.is_finite() || frequency_hz <= 0.0 {
            return;
        }
        let log2 = frequency_hz.log2();
        for voice in &mut self.voices {
            if voice.live && voice.voice_id == voice_id {
                voice.start_log2 = log2;
                voice.target_log2 = log2;
                voice.elapsed_samples = 0;
                voice.total_samples = 0;
                voice.moving = false;
                let slot = &mut self.slots[voice.slot_index];
                if slot.latest_voice_id == voice_id {
                    slot.current_log2 = log2;
                    slot.has_history = true;
                }
            }
        }
    }

    pub fn advance(&mut self, frames: usize) {
        let frames = frames.min(u32::MAX as usize) as u32;
        for voice in &mut self.voices {
            if !voice.live || !voice.moving {
                continue;
            }
            voice.elapsed_samples = voice
                .elapsed_samples
                .saturating_add(frames)
                .min(voice.total_samples);
            if self.slots[voice.slot_index].latest_voice_id == voice.voice_id {
                self.slots[voice.slot_index].current_log2 = voice.frequency().log2();
            }
            if voice.elapsed_samples >= voice.total_samples {
                voice.moving = false;
            }
        }
    }

    pub fn for_each_moving(&self, mut visit: impl FnMut(u64, f32)) {
        for voice in &self.voices {
            if voice.live && voice.moving {
                visit(voice.voice_id, voice.frequency());
            }
        }
    }

    pub fn clear(&mut self) {
        self.slots = [SlotState::EMPTY; MAX_SLIDE_VOICES];
        self.voices = [VoiceState::EMPTY; MAX_SLIDE_VOICES];
    }
}

fn shape(progress: f32, curve: SlideCurve) -> f32 {
    const EXPONENT: f32 = 3.0;
    match curve {
        SlideCurve::Linear => progress,
        SlideCurve::Exponential => (1.0 - (-EXPONENT * progress).exp()) / (1.0 - (-EXPONENT).exp()),
        SlideCurve::InverseExponential => {
            let remaining = 1.0 - progress;
            1.0 - (1.0 - (-EXPONENT * remaining).exp()) / (1.0 - (-EXPONENT).exp())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timed(curve: SlideCurve, trigger: SlideTrigger) -> SlideSettings {
        SlideSettings {
            travel: SlideTravel::Time {
                milliseconds: 100.0,
            },
            trigger,
            curve,
        }
    }

    #[test]
    fn property_overrides_resolve_independently() {
        let parent = SlideSettings {
            travel: SlideTravel::Time {
                milliseconds: 180.0,
            },
            trigger: SlideTrigger::Legato,
            curve: SlideCurve::Linear,
        };
        let child = SlideOverride {
            travel: None,
            trigger: Some(SlideTrigger::Always),
            curve: Some(SlideCurve::InverseExponential),
        }
        .resolve(parent);
        assert_eq!(child.travel, parent.travel);
        assert_eq!(child.trigger, SlideTrigger::Always);
        assert_eq!(child.curve, SlideCurve::InverseExponential);
    }

    #[test]
    fn legato_requires_overlap_but_always_uses_slot_history() {
        let slot = SlideSlot::new(SlideRole::Input, 0);
        let mut runtime = SlideRuntime::new();
        assert_eq!(
            runtime.note_on(
                1,
                slot,
                440.0,
                timed(SlideCurve::Linear, SlideTrigger::Legato),
                48_000.0
            ),
            440.0
        );
        runtime.note_off(1);
        assert_eq!(
            runtime.note_on(
                2,
                slot,
                880.0,
                timed(SlideCurve::Linear, SlideTrigger::Legato),
                48_000.0
            ),
            880.0
        );
        runtime.note_off(2);
        assert_eq!(
            runtime.note_on(
                3,
                slot,
                440.0,
                timed(SlideCurve::Linear, SlideTrigger::Always),
                48_000.0
            ),
            880.0
        );
    }

    #[test]
    fn duplicate_voice_ids_release_exactly() {
        let slot = SlideSlot::new(SlideRole::Harmony, 2);
        let settings = timed(SlideCurve::Linear, SlideTrigger::Legato);
        let mut runtime = SlideRuntime::new();
        runtime.note_on(10, slot, 440.0, settings, 48_000.0);
        runtime.note_on(11, slot, 880.0, settings, 48_000.0);
        runtime.note_off(10);
        runtime.advance(2_400);
        let mut moving = [None; MAX_SLIDE_VOICES];
        let mut len = 0;
        runtime.for_each_moving(|id, frequency| {
            moving[len] = Some((id, frequency));
            len += 1;
        });
        assert_eq!(len, 1);
        assert_eq!(moving[0].unwrap().0, 11);
    }

    #[test]
    fn curves_have_exact_endpoints_and_distinct_midpoints() {
        for curve in [
            SlideCurve::Linear,
            SlideCurve::Exponential,
            SlideCurve::InverseExponential,
        ] {
            assert!((shape(0.0, curve) - 0.0).abs() < 1.0e-6);
            assert!((shape(1.0, curve) - 1.0).abs() < 1.0e-6);
        }
        assert!(shape(0.5, SlideCurve::Exponential) > 0.5);
        assert!(shape(0.5, SlideCurve::InverseExponential) < 0.5);
    }

    #[test]
    fn constant_rate_duration_scales_with_interval() {
        let travel = SlideTravel::Rate {
            semitones_per_second: 12.0,
        };
        assert_eq!(travel.samples(440.0, 880.0, 48_000.0), Some(48_000));
        assert_eq!(travel.samples(440.0, 1_760.0, 48_000.0), Some(96_000));
    }

    #[test]
    fn runtime_path_does_not_allocate() {
        assert_no_alloc::assert_no_alloc(|| {
            let slot = SlideSlot::new(SlideRole::Harmony, 3);
            let mut runtime = SlideRuntime::new();
            let settings = timed(SlideCurve::Exponential, SlideTrigger::Always);
            runtime.note_on(1, slot, 440.0, settings, 48_000.0);
            runtime.note_off(1);
            runtime.note_on(2, slot, 660.0, settings, 48_000.0);
            runtime.advance(64);
            runtime.for_each_moving(|_, frequency| assert!(frequency.is_finite()));
            runtime.note_off(2);
            runtime.clear();
        });
    }

    #[test]
    fn trajectory_reaches_exact_target() {
        let slot = SlideSlot::new(SlideRole::Input, 0);
        let mut runtime = SlideRuntime::new();
        let settings = timed(SlideCurve::Exponential, SlideTrigger::Always);
        runtime.note_on(1, slot, 440.0, settings, 48_000.0);
        runtime.note_off(1);
        runtime.note_on(2, slot, 880.0, settings, 48_000.0);
        runtime.advance(4_800);
        let voice = runtime
            .voices
            .iter()
            .find(|voice| voice.live && voice.voice_id == 2)
            .unwrap();
        assert!(!voice.moving);
        assert!((voice.frequency() - 880.0).abs() < 1.0e-3);
    }
}
