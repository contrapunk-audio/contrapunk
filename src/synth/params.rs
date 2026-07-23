//! Runtime-tweakable synth parameters, lock-free.
//!
//! All parameters are atomics scaled to integer storage so the audio
//! callback can read them without locks. Ranges are documented on
//! each getter.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::{mpsc, Arc, Mutex};

/// Oscillator waveform. Stored as a u8 in [`SynthParams::waveform`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Waveform {
    Sine = 0,
    Saw = 1,
    Square = 2,
    Triangle = 3,
}

impl Waveform {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Waveform::Saw,
            2 => Waveform::Square,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        }
    }
}

/// Stable identity assigned before a synth event enters the audio path.
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

/// Voice events pushed from the router thread into the audio thread.
#[derive(Clone, Copy, Debug)]
pub enum SynthEvent {
    NoteOn {
        voice_id: SynthVoiceId,
        midi_anchor: u8,
        frequency_hz: f32,
        velocity: u8,
        mix_group: u8,
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

    pub const fn note_off(voice_id: SynthVoiceId) -> Self {
        Self::NoteOff { voice_id }
    }
}

/// Fixed router-to-synth queue bound. Overflow is non-blocking and raises
/// a shared fault so the audio side can panic instead of stranding a voice.
pub const SYNTH_EVENT_QUEUE_CAPACITY: usize = 1024;
const ROUTER_VOICE_ID_PREFIX: u64 = 1 << 62;

#[derive(Clone, Copy)]
struct SynthOwner {
    voice_id: SynthVoiceId,
    midi_anchor: u8,
    mix_group: u8,
}

struct SynthOwners {
    next_id: u64,
    active: Vec<SynthOwner>,
}

impl SynthOwners {
    fn new() -> Self {
        Self {
            next_id: 0,
            active: Vec::with_capacity(SYNTH_EVENT_QUEUE_CAPACITY),
        }
    }

    fn allocate(&mut self, midi_anchor: u8, mix_group: u8) -> Option<SynthVoiceId> {
        if midi_anchor >= 128 || self.active.len() == self.active.capacity() {
            return None;
        }
        let voice_id = SynthVoiceId::new(ROUTER_VOICE_ID_PREFIX | self.next_id);
        self.next_id = self.next_id.wrapping_add(1) & (ROUTER_VOICE_ID_PREFIX - 1);
        self.active.push(SynthOwner {
            voice_id,
            midi_anchor,
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
    owners: Arc<Mutex<SynthOwners>>,
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
            owners: Arc::new(Mutex::new(SynthOwners::new())),
        },
        SynthEventReceiver { rx, fault },
    )
}

impl SynthEventSender {
    /// Non-blocking send. A full queue marks the audio side for panic.
    pub fn send(&self, event: SynthEvent) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        if matches!(event, SynthEvent::AllNotesOff) {
            let mut owners = self
                .owners
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            owners.active.clear();
            return self.try_send(event);
        }
        self.try_send(event)
    }

    /// Assign a bounded router identity before entering the audio path.
    pub fn note_on(
        &self,
        midi_anchor: u8,
        velocity: u8,
        mix_group: u8,
    ) -> Result<SynthVoiceId, mpsc::TrySendError<SynthEvent>> {
        let mut owners = self
            .owners
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(voice_id) = owners.allocate(midi_anchor, mix_group) else {
            self.fault.store(true, Ordering::Release);
            return Err(mpsc::TrySendError::Full(SynthEvent::AllNotesOff));
        };
        let event = SynthEvent::note_on(
            voice_id,
            midi_anchor,
            midi_to_freq(midi_anchor),
            velocity,
            mix_group,
        );
        if let Err(error) = self.try_send(event) {
            owners.active.clear();
            return Err(error);
        }
        Ok(voice_id)
    }

    /// Release the oldest matching owner. `MIX_GROUP_ALL` releases every role.
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

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2f32.powf((note as f32 - 69.0) / 12.0)
}

impl SynthEventReceiver {
    pub fn try_recv(&self) -> Result<SynthEvent, mpsc::TryRecvError> {
        self.rx.try_recv()
    }

    pub fn recv(&self) -> Result<SynthEvent, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn take_fault(&self) -> bool {
        self.fault.swap(false, Ordering::AcqRel)
    }

    pub fn fault_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.fault)
    }
}

/// Per-role mix groups used by the native performance mixer.
pub const MIX_GROUP_COUNT: usize = 4;
pub const MIX_GROUP_ALL: u8 = u8::MAX;

/// Shared parameter store. Cloneable `Arc<SynthParams>` for UI + audio.
///
/// Fixed-point encodings:
/// - Times (attack/decay/release) stored in milliseconds (AtomicU32).
/// - Linear 0..1 values (sustain level, resonance, master gain) stored
///   as parts per thousand (0..1000 → 0.0..1.0) in AtomicU32.
/// - Cutoff stored in Hz (AtomicU32).
pub struct SynthParams {
    waveform: AtomicU8,
    attack_ms: AtomicU32,
    decay_ms: AtomicU32,
    sustain_ppt: AtomicU32,
    release_ms: AtomicU32,
    cutoff_hz: AtomicU32,
    resonance_ppt: AtomicU32,
    master_gain_ppt: AtomicU32,
    mix_gain_ppt: [AtomicU32; MIX_GROUP_COUNT],
    enabled: AtomicBool,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            waveform: AtomicU8::new(Waveform::Sine as u8),
            attack_ms: AtomicU32::new(5),
            decay_ms: AtomicU32::new(120),
            sustain_ppt: AtomicU32::new(700), // 0.70
            release_ms: AtomicU32::new(250),
            cutoff_hz: AtomicU32::new(6000),
            resonance_ppt: AtomicU32::new(200),   // 0.20
            master_gain_ppt: AtomicU32::new(250), // 0.25 — conservative
            mix_gain_ppt: std::array::from_fn(|_| AtomicU32::new(1000)),
            enabled: AtomicBool::new(true),
        }
    }
}

impl SynthParams {
    pub fn new() -> Self {
        Self::default()
    }

    // ─── Readers (audio-thread hot path) ─────────────────────────

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    pub fn waveform(&self) -> Waveform {
        Waveform::from_u8(self.waveform.load(Ordering::Relaxed))
    }
    pub fn attack_secs(&self) -> f32 {
        self.attack_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn decay_secs(&self) -> f32 {
        self.decay_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn sustain_level(&self) -> f32 {
        self.sustain_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn release_secs(&self) -> f32 {
        self.release_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn cutoff_hz(&self) -> f32 {
        self.cutoff_hz.load(Ordering::Relaxed) as f32
    }
    pub fn resonance(&self) -> f32 {
        self.resonance_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn master_gain(&self) -> f32 {
        self.master_gain_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn mix_gains(&self) -> [f32; MIX_GROUP_COUNT] {
        std::array::from_fn(|i| self.mix_gain_ppt[i].load(Ordering::Relaxed) as f32 / 1000.0)
    }

    // ─── Writers (command thread) ────────────────────────────────

    pub fn set_enabled(&self, v: bool) {
        self.enabled.store(v, Ordering::Relaxed);
    }
    pub fn set_waveform(&self, w: Waveform) {
        self.waveform.store(w as u8, Ordering::Relaxed);
    }
    pub fn set_attack_ms(&self, ms: u32) {
        self.attack_ms.store(ms.clamp(1, 5_000), Ordering::Relaxed);
    }
    pub fn set_decay_ms(&self, ms: u32) {
        self.decay_ms.store(ms.clamp(1, 5_000), Ordering::Relaxed);
    }
    pub fn set_sustain_level(&self, v: f32) {
        if v.is_finite() {
            let ppt = (v.clamp(0.0, 1.0) * 1000.0) as u32;
            self.sustain_ppt.store(ppt, Ordering::Relaxed);
        }
    }
    pub fn set_release_ms(&self, ms: u32) {
        self.release_ms
            .store(ms.clamp(1, 10_000), Ordering::Relaxed);
    }
    pub fn set_cutoff_hz(&self, hz: u32) {
        self.cutoff_hz
            .store(hz.clamp(20, 20_000), Ordering::Relaxed);
    }
    pub fn set_resonance(&self, v: f32) {
        if v.is_finite() {
            let ppt = (v.clamp(0.0, 1.0) * 1000.0) as u32;
            self.resonance_ppt.store(ppt, Ordering::Relaxed);
        }
    }
    pub fn set_master_gain(&self, v: f32) {
        if v.is_finite() {
            let ppt = (v.clamp(0.0, 1.0) * 1000.0) as u32;
            self.master_gain_ppt.store(ppt, Ordering::Relaxed);
        }
    }
    pub fn set_mix_gain(&self, group: usize, v: f32) {
        if !v.is_finite() {
            return;
        }
        if let Some(gain) = self.mix_gain_ppt.get(group) {
            let ppt = (v.clamp(0.0, 1.0) * 1000.0) as u32;
            gain.store(ppt, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_atomic_controls_preserve_last_valid_state() {
        let params = SynthParams::new();
        params.set_sustain_level(0.2);
        params.set_resonance(0.3);
        params.set_master_gain(0.4);
        params.set_mix_gain(1, 0.5);

        params.set_sustain_level(f32::NAN);
        params.set_resonance(f32::INFINITY);
        params.set_master_gain(f32::NEG_INFINITY);
        params.set_mix_gain(1, f32::NAN);

        assert_eq!(params.sustain_level(), 0.2);
        assert_eq!(params.resonance(), 0.3);
        assert_eq!(params.master_gain(), 0.4);
        assert_eq!(params.mix_gains()[1], 0.5);
    }

    #[test]
    fn sender_assigns_distinct_ids_and_releases_exact_or_stale_anchor_owners() {
        let (tx, rx) = synth_event_channel();
        let first = tx.note_on(69, 100, 0).unwrap();
        let second = tx.note_on(69, 100, 0).unwrap();
        assert_ne!(first, second);

        for expected in [first, second] {
            assert!(matches!(
                rx.try_recv().unwrap(),
                SynthEvent::NoteOn {
                    voice_id,
                    midi_anchor: 69,
                    frequency_hz: 440.0,
                    mix_group: 0,
                    ..
                } if voice_id == expected
            ));
        }

        tx.note_off(69, 0).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOff { voice_id } if voice_id == first
        ));
        tx.note_off(69, MIX_GROUP_ALL).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            SynthEvent::NoteOff { voice_id } if voice_id == second
        ));
        assert!(rx.try_recv().is_err());

        tx.note_on(72, 100, 1).unwrap();
        let _ = rx.try_recv().unwrap();
        tx.send(SynthEvent::AllNotesOff).unwrap();
        assert!(matches!(rx.try_recv().unwrap(), SynthEvent::AllNotesOff));
        tx.note_off(72, MIX_GROUP_ALL).unwrap();
        assert!(rx.try_recv().is_err(), "panic must clear adapter ownership");
    }

    #[test]
    fn synth_event_channel_is_bounded_and_marks_overflow() {
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
}
