//! Native routing contract for the fixed-sine Elixir engine.
//!
//! Producers assign stable bounded voice identities before events enter the
//! audio thread. Controls are limited to enable, master gain, and role gains.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc, Mutex};

pub use elixir_core::VoiceRole;

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
    },
    Retune {
        voice_id: SynthVoiceId,
        frequency_hz: f32,
    },
    NoteOff {
        voice_id: SynthVoiceId,
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
        Self::NoteOn {
            voice_id,
            midi_anchor,
            frequency_hz,
            velocity,
            mix_group,
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
        });
        Some(voice_id)
    }

    fn release(&mut self, midi_anchor: u8, mix_group: u8) -> Option<SynthVoiceId> {
        let index = self.active.iter().position(|owner| {
            owner.midi_anchor == midi_anchor
                && (mix_group == MIX_GROUP_ALL || owner.mix_group == mix_group)
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
        if midi_anchor >= 128 || velocity == 0 || !frequency_hz.is_finite() || frequency_hz <= 0.0 {
            self.fault.store(true, Ordering::Release);
            return Err(mpsc::TrySendError::Full(SynthEvent::AllNotesOff));
        }
        let mut owners = self
            .owners
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(voice_id) = owners.allocate(midi_anchor, frequency_hz, mix_group) else {
            self.fault.store(true, Ordering::Release);
            return Err(mpsc::TrySendError::Full(SynthEvent::AllNotesOff));
        };
        let sounding_frequency = if self.compare_standard.load(Ordering::Acquire) {
            elixir_core::util::midi_to_freq(midi_anchor)
        } else {
            frequency_hz
        };
        let event = SynthEvent::note_on(
            voice_id,
            midi_anchor,
            sounding_frequency,
            velocity,
            mix_group,
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
        while let Some(voice_id) = owners.release(midi_anchor, mix_group) {
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

pub struct SynthParams {
    enabled: AtomicBool,
    master_gain_ppt: AtomicU32,
    mix_gain_ppt: [AtomicU32; MIX_GROUP_COUNT],
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            master_gain_ppt: AtomicU32::new(250),
            mix_gain_ppt: std::array::from_fn(|_| AtomicU32::new(1000)),
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
