//! Runtime-tweakable synth parameters, lock-free.
//!
//! All parameters are atomics scaled to integer storage so the audio
//! callback can read them without locks. Ranges are documented on
//! each getter.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::{mpsc, Arc};

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

/// MIDI-ish events pushed from the router thread into the audio thread.
#[derive(Clone, Copy, Debug)]
pub enum SynthEvent {
    NoteOn {
        note: u8,
        velocity: u8,
        mix_group: u8,
    },
    NoteOff {
        note: u8,
        mix_group: u8,
    },
    AllNotesOff,
}

/// Fixed router-to-synth queue bound. Overflow is non-blocking and raises
/// a shared fault so the audio side can panic instead of stranding a voice.
pub const SYNTH_EVENT_QUEUE_CAPACITY: usize = 1024;

#[derive(Clone)]
pub struct SynthEventSender {
    tx: mpsc::SyncSender<SynthEvent>,
    fault: Arc<AtomicBool>,
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
        },
        SynthEventReceiver { rx, fault },
    )
}

impl SynthEventSender {
    /// Non-blocking send. A full queue marks the audio side for panic.
    pub fn send(&self, event: SynthEvent) -> Result<(), mpsc::TrySendError<SynthEvent>> {
        self.tx.try_send(event).inspect_err(|error| {
            if matches!(error, mpsc::TrySendError::Full(_)) {
                self.fault.store(true, Ordering::Release);
            }
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
