//! Native routing contract for the role-aware Elixir foundations engine.
//!
//! Producers assign stable bounded voice identities before events enter the
//! audio thread. Sound parameters use lock-free scalar snapshots.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::slide::{SlideCurve, SlideRole, SlideSettings, SlideSlot, SlideTravel, SlideTrigger};

pub use elixir_core::{
    role_param, AmpEnvelope, CombineMode, HarmonicRecipe, RolePatch, SecondaryOscillator, Vibrato,
    VoiceRole, PARTIAL_COUNT,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CombineModeState {
    PrimaryOnly,
    Add,
    Ring,
}

impl From<CombineMode> for CombineModeState {
    fn from(mode: CombineMode) -> Self {
        match mode {
            CombineMode::PrimaryOnly => Self::PrimaryOnly,
            CombineMode::Add => Self::Add,
            CombineMode::Ring => Self::Ring,
        }
    }
}

impl From<CombineModeState> for CombineMode {
    fn from(mode: CombineModeState) -> Self {
        match mode {
            CombineModeState::PrimaryOnly => Self::PrimaryOnly,
            CombineModeState::Add => Self::Add,
            CombineModeState::Ring => Self::Ring,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct HarmonicRecipeState {
    pub amplitudes: [f32; PARTIAL_COUNT],
    pub phases: [f32; PARTIAL_COUNT],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct SecondaryOscillatorState {
    pub mode: CombineModeState,
    pub semitones: f32,
    pub fine_cents: f32,
    pub phase: f32,
    pub level: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct AmpEnvelopeState {
    pub attack_secs: f32,
    pub decay_secs: f32,
    pub sustain_level: f32,
    pub release_secs: f32,
    pub velocity_sensitivity: f32,
    pub expression_sensitivity: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct VibratoState {
    pub rate_hz: f32,
    pub depth_cents: f32,
    pub mod_wheel_depth_cents: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct RolePatchState {
    pub harmonics: HarmonicRecipeState,
    pub secondary: SecondaryOscillatorState,
    pub envelope: AmpEnvelopeState,
    pub vibrato: VibratoState,
}

impl From<RolePatch> for RolePatchState {
    fn from(patch: RolePatch) -> Self {
        Self {
            harmonics: HarmonicRecipeState {
                amplitudes: patch.harmonics.amplitudes,
                phases: patch.harmonics.phases,
            },
            secondary: SecondaryOscillatorState {
                mode: patch.secondary.mode.into(),
                semitones: patch.secondary.semitones,
                fine_cents: patch.secondary.fine_cents,
                phase: patch.secondary.phase,
                level: patch.secondary.level,
            },
            envelope: AmpEnvelopeState {
                attack_secs: patch.envelope.attack_secs,
                decay_secs: patch.envelope.decay_secs,
                sustain_level: patch.envelope.sustain_level,
                release_secs: patch.envelope.release_secs,
                velocity_sensitivity: patch.envelope.velocity_sensitivity,
                expression_sensitivity: patch.envelope.expression_sensitivity,
            },
            vibrato: VibratoState {
                rate_hz: patch.vibrato.rate_hz,
                depth_cents: patch.vibrato.depth_cents,
                mod_wheel_depth_cents: patch.vibrato.mod_wheel_depth_cents,
            },
        }
    }
}

impl From<RolePatchState> for RolePatch {
    fn from(patch: RolePatchState) -> Self {
        Self {
            harmonics: HarmonicRecipe {
                amplitudes: patch.harmonics.amplitudes,
                phases: patch.harmonics.phases,
            },
            secondary: SecondaryOscillator {
                mode: patch.secondary.mode.into(),
                semitones: patch.secondary.semitones,
                fine_cents: patch.secondary.fine_cents,
                phase: patch.secondary.phase,
                level: patch.secondary.level,
            },
            envelope: AmpEnvelope {
                attack_secs: patch.envelope.attack_secs,
                decay_secs: patch.envelope.decay_secs,
                sustain_level: patch.envelope.sustain_level,
                release_secs: patch.envelope.release_secs,
                velocity_sensitivity: patch.envelope.velocity_sensitivity,
                expression_sensitivity: patch.envelope.expression_sensitivity,
            },
            vibrato: Vibrato {
                rate_hz: patch.vibrato.rate_hz,
                depth_cents: patch.vibrato.depth_cents,
                mod_wheel_depth_cents: patch.vibrato.mod_wheel_depth_cents,
            },
        }
        .sanitized()
    }
}

impl Default for RolePatchState {
    fn default() -> Self {
        RolePatch::sine().into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SynthVoiceId(u64);

impl SynthVoiceId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SynthEvent {
    NoteOn {
        voice_id: SynthVoiceId,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        mix_group: u8,
        slide_slot: SlideSlot,
        slide: SlideSettings,
    },
    Retune {
        voice_id: SynthVoiceId,
        frequency_hz: f32,
    },
    NoteOff {
        voice_id: SynthVoiceId,
    },
    SustainPedal {
        on: bool,
    },
    PitchBend {
        cents: f32,
    },
    Expression {
        value: f32,
    },
    ModWheel {
        value: f32,
    },
    AllNotesOff,
}

impl SynthEvent {
    pub const fn note_on(
        voice_id: SynthVoiceId,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        mix_group: u8,
    ) -> Self {
        Self::note_on_with_slide(
            voice_id,
            midi_anchor,
            frequency_hz,
            velocity,
            mix_group,
            SlideSlot::new(SlideRole::Input, 0),
            SlideSettings {
                travel: SlideTravel::Off,
                trigger: SlideTrigger::Legato,
                curve: SlideCurve::Linear,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub const fn note_on_with_slide(
        voice_id: SynthVoiceId,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        mix_group: u8,
        slide_slot: SlideSlot,
        slide: SlideSettings,
    ) -> Self {
        Self::NoteOn {
            voice_id,
            midi_anchor,
            frequency_hz,
            velocity,
            mix_group,
            slide_slot,
            slide,
        }
    }

    pub const fn retune(voice_id: SynthVoiceId, frequency_hz: f32) -> Self {
        Self::Retune {
            voice_id,
            frequency_hz,
        }
    }

    pub const fn note_off(voice_id: SynthVoiceId) -> Self {
        Self::NoteOff { voice_id }
    }

    pub const fn sustain_pedal(on: bool) -> Self {
        Self::SustainPedal { on }
    }

    pub const fn pitch_bend(cents: f32) -> Self {
        Self::PitchBend { cents }
    }

    pub const fn expression(value: f32) -> Self {
        Self::Expression { value }
    }

    pub const fn mod_wheel(value: f32) -> Self {
        Self::ModWheel { value }
    }
}

pub const SYNTH_EVENT_QUEUE_CAPACITY: usize = 1024;
pub const MIX_GROUP_COUNT: usize = 4;
pub const MIX_GROUP_ALL: u8 = u8::MAX;
const ROUTER_VOICE_ID_PREFIX: u64 = 1 << 62;

#[derive(Clone, Copy)]
struct Owner {
    voice_id: SynthVoiceId,
    midi_anchor: u8,
    frequency_hz: f32,
    mix_group: u8,
    slide_slot: SlideSlot,
}

struct Owners {
    next_id: u64,
    active: Vec<Owner>,
}

impl Owners {
    fn new() -> Self {
        Self {
            next_id: 0,
            active: Vec::with_capacity(SYNTH_EVENT_QUEUE_CAPACITY),
        }
    }

    fn allocate(
        &mut self,
        midi_anchor: u8,
        frequency_hz: f32,
        mix_group: u8,
        slide_slot: SlideSlot,
    ) -> Option<SynthVoiceId> {
        if midi_anchor >= 128 || self.active.len() == self.active.capacity() {
            return None;
        }
        let voice_id = SynthVoiceId::new(ROUTER_VOICE_ID_PREFIX | self.next_id);
        self.next_id = self.next_id.wrapping_add(1) & (ROUTER_VOICE_ID_PREFIX - 1);
        self.active.push(Owner {
            voice_id,
            midi_anchor,
            frequency_hz,
            mix_group,
            slide_slot,
        });
        Some(voice_id)
    }

    fn release(
        &mut self,
        midi_anchor: u8,
        mix_group: u8,
        slide_slot: Option<SlideSlot>,
    ) -> Option<SynthVoiceId> {
        let index = self.active.iter().position(|owner| {
            owner.midi_anchor == midi_anchor
                && (mix_group == MIX_GROUP_ALL || owner.mix_group == mix_group)
                && slide_slot.is_none_or(|slot| owner.slide_slot == slot)
        })?;
        Some(self.active.remove(index).voice_id)
    }
}

#[derive(Clone)]
pub struct SynthEventSender {
    tx: mpsc::SyncSender<SynthEvent>,
    fault: Arc<AtomicBool>,
    owners: Arc<Mutex<Owners>>,
    compare_standard: Arc<AtomicBool>,
}

pub struct SynthEventReceiver {
    rx: mpsc::Receiver<SynthEvent>,
    fault: Arc<AtomicBool>,
}

pub fn synth_event_channel() -> (SynthEventSender, SynthEventReceiver) {
    let (tx, rx) = mpsc::sync_channel(SYNTH_EVENT_QUEUE_CAPACITY);
    let fault = Arc::new(AtomicBool::new(false));
    (
        SynthEventSender {
            tx,
            fault: Arc::clone(&fault),
            owners: Arc::new(Mutex::new(Owners::new())),
            compare_standard: Arc::new(AtomicBool::new(false)),
        },
        SynthEventReceiver { rx, fault },
    )
}

impl SynthEventSender {
    pub fn send(&self, event: SynthEvent) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        if matches!(event, SynthEvent::AllNotesOff) {
            self.owners
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .active
                .clear();
        }
        self.try_send(event)
    }

    pub fn note_on(
        &self,
        midi_anchor: u8,
        velocity: u8,
        mix_group: u8,
    ) -> Result<SynthVoiceId, mpsc::TrySendError<SynthEvent>> {
        self.note_on_exact(
            midi_anchor,
            elixir_core::util::midi_to_freq(midi_anchor),
            velocity,
            mix_group,
        )
    }

    pub fn note_on_exact(
        &self,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        mix_group: u8,
    ) -> Result<SynthVoiceId, mpsc::TrySendError<SynthEvent>> {
        self.note_on_exact_with_slide(
            midi_anchor,
            frequency_hz,
            velocity,
            mix_group,
            SlideSlot::new(SlideRole::Input, 0),
            SlideSettings::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn note_on_exact_with_slide(
        &self,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        mix_group: u8,
        slide_slot: SlideSlot,
        slide: SlideSettings,
    ) -> Result<SynthVoiceId, mpsc::TrySendError<SynthEvent>> {
        if midi_anchor >= 128 || velocity == 0 || !frequency_hz.is_finite() || frequency_hz <= 0.0 {
            self.fault.store(true, Ordering::Release);
            return Err(mpsc::TrySendError::Full(SynthEvent::AllNotesOff));
        }
        let mut owners = self
            .owners
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(voice_id) = owners.allocate(midi_anchor, frequency_hz, mix_group, slide_slot)
        else {
            self.fault.store(true, Ordering::Release);
            return Err(mpsc::TrySendError::Full(SynthEvent::AllNotesOff));
        };
        let sounding_frequency = if self.compare_standard.load(Ordering::Acquire) {
            elixir_core::util::midi_to_freq(midi_anchor)
        } else {
            frequency_hz
        };
        let event = SynthEvent::note_on_with_slide(
            voice_id,
            midi_anchor,
            sounding_frequency,
            velocity,
            mix_group,
            slide_slot,
            slide,
        );
        if let Err(error) = self.try_send(event) {
            owners.active.clear();
            return Err(error);
        }
        Ok(voice_id)
    }

    pub fn set_compare_standard(
        &self,
        enabled: bool,
    ) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        if self.compare_standard.swap(enabled, Ordering::AcqRel) == enabled {
            return Ok(());
        }
        let mut owners = self
            .owners
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for owner in &owners.active {
            let frequency_hz = if enabled {
                elixir_core::util::midi_to_freq(owner.midi_anchor)
            } else {
                owner.frequency_hz
            };
            if let Err(error) = self.try_send(SynthEvent::retune(owner.voice_id, frequency_hz)) {
                owners.active.clear();
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn note_off(
        &self,
        midi_anchor: u8,
        mix_group: u8,
    ) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        let mut owners = self
            .owners
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        while let Some(voice_id) = owners.release(midi_anchor, mix_group, None) {
            if let Err(error) = self.try_send(SynthEvent::note_off(voice_id)) {
                owners.active.clear();
                return Err(error);
            }
            if mix_group != MIX_GROUP_ALL {
                break;
            }
        }
        Ok(())
    }

    pub fn note_off_slot(
        &self,
        midi_anchor: u8,
        mix_group: u8,
        slide_slot: SlideSlot,
    ) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        let mut owners = self
            .owners
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(voice_id) = owners.release(midi_anchor, mix_group, Some(slide_slot)) else {
            return Ok(());
        };
        if let Err(error) = self.try_send(SynthEvent::note_off(voice_id)) {
            owners.active.clear();
            return Err(error);
        }
        Ok(())
    }

    pub fn pitch_bend(&self, cents: f32) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        self.try_send(SynthEvent::pitch_bend(cents))
    }

    pub fn expression(&self, value: f32) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        self.try_send(SynthEvent::expression(value))
    }

    pub fn mod_wheel(&self, value: f32) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        self.try_send(SynthEvent::mod_wheel(value))
    }

    fn try_send(&self, event: SynthEvent) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        self.tx.try_send(event).inspect_err(|_| {
            self.fault.store(true, Ordering::Release);
        })
    }
}

impl SynthEventReceiver {
    pub fn try_recv(&self) -> Result<SynthEvent, mpsc::TryRecvError> {
        self.rx.try_recv()
    }

    pub fn recv(&self) -> Result<SynthEvent, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn fault_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.fault)
    }
}

struct AtomicFloat(AtomicU32);

impl AtomicFloat {
    fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }

    fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    fn store(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed);
    }
}

struct RolePatchParams {
    amplitudes: [AtomicFloat; PARTIAL_COUNT],
    phases: [AtomicFloat; PARTIAL_COUNT],
    combine_mode: AtomicU8,
    secondary_semitones: AtomicFloat,
    secondary_fine_cents: AtomicFloat,
    secondary_phase: AtomicFloat,
    secondary_level: AtomicFloat,
    attack_secs: AtomicFloat,
    decay_secs: AtomicFloat,
    sustain_level: AtomicFloat,
    release_secs: AtomicFloat,
    velocity_sensitivity: AtomicFloat,
    expression_sensitivity: AtomicFloat,
    vibrato_rate_hz: AtomicFloat,
    vibrato_depth_cents: AtomicFloat,
    mod_wheel_depth_cents: AtomicFloat,
}

impl RolePatchParams {
    fn new(patch: RolePatch) -> Self {
        let patch = patch.sanitized();
        Self {
            amplitudes: std::array::from_fn(|index| {
                AtomicFloat::new(patch.harmonics.amplitudes[index])
            }),
            phases: std::array::from_fn(|index| AtomicFloat::new(patch.harmonics.phases[index])),
            combine_mode: AtomicU8::new(patch.secondary.mode as u8),
            secondary_semitones: AtomicFloat::new(patch.secondary.semitones),
            secondary_fine_cents: AtomicFloat::new(patch.secondary.fine_cents),
            secondary_phase: AtomicFloat::new(patch.secondary.phase),
            secondary_level: AtomicFloat::new(patch.secondary.level),
            attack_secs: AtomicFloat::new(patch.envelope.attack_secs),
            decay_secs: AtomicFloat::new(patch.envelope.decay_secs),
            sustain_level: AtomicFloat::new(patch.envelope.sustain_level),
            release_secs: AtomicFloat::new(patch.envelope.release_secs),
            velocity_sensitivity: AtomicFloat::new(patch.envelope.velocity_sensitivity),
            expression_sensitivity: AtomicFloat::new(patch.envelope.expression_sensitivity),
            vibrato_rate_hz: AtomicFloat::new(patch.vibrato.rate_hz),
            vibrato_depth_cents: AtomicFloat::new(patch.vibrato.depth_cents),
            mod_wheel_depth_cents: AtomicFloat::new(patch.vibrato.mod_wheel_depth_cents),
        }
    }

    fn load(&self) -> RolePatch {
        RolePatch {
            harmonics: HarmonicRecipe {
                amplitudes: std::array::from_fn(|index| self.amplitudes[index].load()),
                phases: std::array::from_fn(|index| self.phases[index].load()),
            },
            secondary: SecondaryOscillator {
                mode: CombineMode::from_index(self.combine_mode.load(Ordering::Relaxed))
                    .unwrap_or_default(),
                semitones: self.secondary_semitones.load(),
                fine_cents: self.secondary_fine_cents.load(),
                phase: self.secondary_phase.load(),
                level: self.secondary_level.load(),
            },
            envelope: AmpEnvelope {
                attack_secs: self.attack_secs.load(),
                decay_secs: self.decay_secs.load(),
                sustain_level: self.sustain_level.load(),
                release_secs: self.release_secs.load(),
                velocity_sensitivity: self.velocity_sensitivity.load(),
                expression_sensitivity: self.expression_sensitivity.load(),
            },
            vibrato: Vibrato {
                rate_hz: self.vibrato_rate_hz.load(),
                depth_cents: self.vibrato_depth_cents.load(),
                mod_wheel_depth_cents: self.mod_wheel_depth_cents.load(),
            },
        }
        .sanitized()
    }

    fn store(&self, patch: RolePatch) {
        let patch = patch.sanitized();
        for index in 0..PARTIAL_COUNT {
            self.amplitudes[index].store(patch.harmonics.amplitudes[index]);
            self.phases[index].store(patch.harmonics.phases[index]);
        }
        self.combine_mode
            .store(patch.secondary.mode as u8, Ordering::Relaxed);
        self.secondary_semitones.store(patch.secondary.semitones);
        self.secondary_fine_cents.store(patch.secondary.fine_cents);
        self.secondary_phase.store(patch.secondary.phase);
        self.secondary_level.store(patch.secondary.level);
        self.attack_secs.store(patch.envelope.attack_secs);
        self.decay_secs.store(patch.envelope.decay_secs);
        self.sustain_level.store(patch.envelope.sustain_level);
        self.release_secs.store(patch.envelope.release_secs);
        self.velocity_sensitivity
            .store(patch.envelope.velocity_sensitivity);
        self.expression_sensitivity
            .store(patch.envelope.expression_sensitivity);
        self.vibrato_rate_hz.store(patch.vibrato.rate_hz);
        self.vibrato_depth_cents.store(patch.vibrato.depth_cents);
        self.mod_wheel_depth_cents
            .store(patch.vibrato.mod_wheel_depth_cents);
    }
}

pub struct SynthParams {
    enabled: AtomicBool,
    master_gain_ppt: AtomicU32,
    mix_gain_ppt: [AtomicU32; MIX_GROUP_COUNT],
    role_patches: [RolePatchParams; MIX_GROUP_COUNT],
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            master_gain_ppt: AtomicU32::new(250),
            mix_gain_ppt: std::array::from_fn(|_| AtomicU32::new(1000)),
            role_patches: std::array::from_fn(|_| RolePatchParams::new(RolePatch::sine())),
        }
    }
}

impl SynthParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn master_gain(&self) -> f32 {
        self.master_gain_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }

    pub fn mix_gains(&self) -> [f32; MIX_GROUP_COUNT] {
        std::array::from_fn(|index| {
            self.mix_gain_ppt[index].load(Ordering::Relaxed) as f32 / 1000.0
        })
    }

    pub fn role_patch(&self, group: usize) -> Option<RolePatch> {
        self.role_patches.get(group).map(RolePatchParams::load)
    }

    pub fn role_patches(&self) -> [RolePatch; MIX_GROUP_COUNT] {
        std::array::from_fn(|index| self.role_patches[index].load())
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn set_master_gain(&self, gain: f32) {
        if gain.is_finite() {
            self.master_gain_ppt
                .store((gain.clamp(0.0, 1.0) * 1000.0) as u32, Ordering::Relaxed);
        }
    }

    pub fn set_mix_gain(&self, group: usize, gain: f32) {
        if !gain.is_finite() {
            return;
        }
        if let Some(target) = self.mix_gain_ppt.get(group) {
            target.store((gain.clamp(0.0, 1.0) * 1000.0) as u32, Ordering::Relaxed);
        }
    }

    pub fn set_role_patch(&self, group: usize, patch: RolePatch) -> bool {
        let Some(target) = self.role_patches.get(group) else {
            return false;
        };
        target.store(patch);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_notes_receive_distinct_fifo_owners() {
        let (tx, rx) = synth_event_channel();
        let first = tx.note_on(69, 100, 0).unwrap();
        let second = tx.note_on(69, 100, 0).unwrap();
        assert_ne!(first, second);
        let _ = rx.try_recv().unwrap();
        let _ = rx.try_recv().unwrap();
        tx.note_off(69, 0).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOff { voice_id } if voice_id == first
        ));
    }

    #[test]
    fn exact_note_on_preserves_frequency() {
        let (tx, rx) = synth_event_channel();
        let voice_id = tx.note_on_exact(69, 432.0, 100, 0).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOn {
                voice_id: received_id,
                midi_anchor: 69,
                frequency_hz: 432.0,
                velocity: 100,
                mix_group: 0,
                ..
            } if received_id == voice_id
        ));
    }

    #[test]
    fn invalid_exact_frequency_faults_without_adding_ownership() {
        let (tx, rx) = synth_event_channel();
        assert!(matches!(
            tx.note_on_exact(69, f32::NAN, 100, 0),
            Err(mpsc::TrySendError::Full(SynthEvent::AllNotesOff))
        ));
        assert!(rx.fault_flag().load(Ordering::Acquire));
        tx.note_off(69, 0).unwrap();
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn different_exact_frequencies_keep_anchor_fifo_release() {
        let (tx, rx) = synth_event_channel();
        let first = tx.note_on_exact(69, 432.0, 100, 0).unwrap();
        let second = tx.note_on_exact(69, 444.0, 100, 0).unwrap();
        let _ = rx.try_recv().unwrap();
        let _ = rx.try_recv().unwrap();
        tx.note_off(69, 0).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOff { voice_id } if voice_id == first
        ));
        tx.note_off(69, 0).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOff { voice_id } if voice_id == second
        ));
    }

    #[test]
    fn generated_slots_release_equal_pitches_by_exact_owner() {
        let (tx, rx) = synth_event_channel();
        let first_slot = SlideSlot::new(SlideRole::Canon, 0);
        let second_slot = SlideSlot::new(SlideRole::Canon, 1);
        let first = tx
            .note_on_exact_with_slide(69, 440.0, 100, 2, first_slot, SlideSettings::default())
            .unwrap();
        let second = tx
            .note_on_exact_with_slide(69, 440.0, 100, 2, second_slot, SlideSettings::default())
            .unwrap();
        let _ = rx.try_recv().unwrap();
        let _ = rx.try_recv().unwrap();
        tx.note_off_slot(69, 2, second_slot).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOff { voice_id } if voice_id == second
        ));
        tx.note_off_slot(69, 2, first_slot).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOff { voice_id } if voice_id == first
        ));
    }

    #[test]
    fn compare_retunes_existing_and_new_voices_without_changing_owners() {
        let (tx, rx) = synth_event_channel();
        let first = tx.note_on_exact(69, 432.0, 100, 0).unwrap();
        let _ = rx.try_recv().unwrap();
        tx.set_compare_standard(true).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::Retune { voice_id, frequency_hz: 440.0 } if voice_id == first
        ));
        let second = tx.note_on_exact(69, 432.0, 100, 0).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOn { voice_id, frequency_hz: 440.0, .. } if voice_id == second
        ));
        tx.set_compare_standard(false).unwrap();
        for expected in [first, second] {
            assert!(matches!(
                rx.try_recv().unwrap(),
                SynthEvent::Retune { voice_id, frequency_hz: 432.0 } if voice_id == expected
            ));
        }
        tx.note_off(69, 0).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOff { voice_id } if voice_id == first
        ));
    }

    #[test]
    fn queue_is_bounded_and_overflow_faults() {
        let (tx, rx) = synth_event_channel();
        for _ in 0..SYNTH_EVENT_QUEUE_CAPACITY {
            tx.send(SynthEvent::AllNotesOff).unwrap();
        }
        assert!(matches!(
            tx.send(SynthEvent::AllNotesOff),
            Err(mpsc::TrySendError::Full(_))
        ));
        assert!(rx.fault_flag().load(Ordering::Acquire));
    }

    #[test]
    fn performance_controls_keep_order_and_values() {
        let (tx, rx) = synth_event_channel();
        tx.pitch_bend(37.5).unwrap();
        tx.expression(0.4).unwrap();
        tx.mod_wheel(0.75).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::PitchBend { cents: 37.5 }
        ));
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::Expression { value: 0.4 }
        ));
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::ModWheel { value: 0.75 }
        ));
    }

    #[test]
    fn role_patch_atomics_and_wire_state_round_trip_safely() {
        let params = SynthParams::new();
        let mut patch = RolePatch::sine();
        patch.harmonics.amplitudes = [1.0, 0.5, 0.25, 0.0, 0.0, 0.0];
        patch.secondary.mode = CombineMode::Ring;
        patch.secondary.semitones = -12.0;
        patch.envelope = AmpEnvelope::ring_down();
        assert!(params.set_role_patch(2, patch));
        assert_eq!(params.role_patch(2), Some(patch));
        assert!(!params.set_role_patch(4, patch));

        let state = RolePatchState::from(patch);
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(
            RolePatch::from(serde_json::from_str::<RolePatchState>(&json).unwrap()),
            patch
        );
    }

    #[test]
    fn controls_reject_non_finite_values() {
        let params = SynthParams::new();
        params.set_master_gain(0.4);
        params.set_mix_gain(1, 0.5);
        params.set_master_gain(f32::NAN);
        params.set_mix_gain(1, f32::INFINITY);
        assert_eq!(params.master_gain(), 0.4);
        assert_eq!(params.mix_gains()[1], 0.5);
    }
}
