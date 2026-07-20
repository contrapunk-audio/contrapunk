//! Logic does not route MIDI emitted by an audio-effect AU to another track.
//! Audio-to-MIDI effects therefore expose a virtual CoreMIDI source, matching
//! the routing used by established guitar-to-MIDI plugins.

use midir::os::unix::VirtualOutput;
use midir::MidiOutput;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::{HeapProd, HeapRb};
use std::ffi::{c_void, CStr};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

const QUEUE_CAPACITY: usize = 8192;
static NEXT_PORT: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MidiPacket {
    bytes: [u8; 3],
    len: u8,
}

impl MidiPacket {
    fn note_on(channel: u8, note: u8, velocity: f32) -> Self {
        Self::three(
            0x90 | channel.min(15),
            note.min(127),
            unit_to_midi(velocity).max(1),
        )
    }

    fn note_off(channel: u8, note: u8, velocity: f32) -> Self {
        Self::three(
            0x80 | channel.min(15),
            note.min(127),
            unit_to_midi(velocity),
        )
    }

    fn control_change(channel: u8, controller: u8, value: f32) -> Self {
        Self::three(
            0xB0 | channel.min(15),
            controller.min(127),
            unit_to_midi(value),
        )
    }

    fn pitch_bend(channel: u8, value: f32) -> Self {
        let bend = (value.clamp(0.0, 1.0) * 16_383.0).round() as u16;
        Self::three(
            0xE0 | channel.min(15),
            (bend & 0x7f) as u8,
            ((bend >> 7) & 0x7f) as u8,
        )
    }

    fn channel_pressure(channel: u8, pressure: f32) -> Self {
        Self {
            bytes: [0xD0 | channel.min(15), unit_to_midi(pressure), 0],
            len: 2,
        }
    }

    fn three(status: u8, data1: u8, data2: u8) -> Self {
        Self {
            bytes: [status, data1, data2],
            len: 3,
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

fn unit_to_midi(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 127.0).round() as u8
}

pub(crate) struct LogicMidiOutput {
    producer: Option<HeapProd<MidiPacket>>,
    stop: Arc<AtomicBool>,
}

impl LogicMidiOutput {
    pub(crate) fn new() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        if !is_logic_audio_unit_host() {
            return Self {
                producer: None,
                stop,
            };
        }

        let queue = HeapRb::<MidiPacket>::new(QUEUE_CAPACITY);
        let (producer, mut consumer) = queue.split();
        let worker_stop = Arc::clone(&stop);
        let port_number = NEXT_PORT.fetch_add(1, Ordering::Relaxed);
        let port_name = if port_number == 1 {
            "Contrapunk Guitar MIDI Out".to_string()
        } else {
            format!("Contrapunk Guitar MIDI Out {port_number}")
        };

        let started = thread::Builder::new()
            .name("contrapunk-logic-midi".into())
            .spawn(move || {
                let Ok(output) = MidiOutput::new("Contrapunk Guitar") else {
                    return;
                };
                let Ok(mut connection) = output.create_virtual(&port_name) else {
                    return;
                };

                while !worker_stop.load(Ordering::Acquire) {
                    let mut sent = false;
                    while let Some(packet) = consumer.try_pop() {
                        let _ = connection.send(packet.as_slice());
                        sent = true;
                    }
                    if !sent {
                        thread::park_timeout(Duration::from_millis(1));
                    }
                }

                for channel in 0..16 {
                    let _ =
                        connection.send(MidiPacket::control_change(channel, 123, 0.0).as_slice());
                }
            })
            .is_ok();

        Self {
            producer: started.then_some(producer),
            stop,
        }
    }

    pub(crate) fn note_on(&mut self, channel: u8, note: u8, velocity: f32) {
        self.push(MidiPacket::note_on(channel, note, velocity));
    }

    pub(crate) fn note_off(&mut self, channel: u8, note: u8, velocity: f32) {
        self.push(MidiPacket::note_off(channel, note, velocity));
    }

    pub(crate) fn control_change(&mut self, channel: u8, controller: u8, value: f32) {
        self.push(MidiPacket::control_change(channel, controller, value));
    }

    pub(crate) fn pitch_bend(&mut self, channel: u8, value: f32) {
        self.push(MidiPacket::pitch_bend(channel, value));
    }

    pub(crate) fn channel_pressure(&mut self, channel: u8, pressure: f32) {
        self.push(MidiPacket::channel_pressure(channel, pressure));
    }

    pub(crate) fn all_notes_off(&mut self) {
        for channel in 0..16 {
            self.control_change(channel, 120, 0.0);
            self.control_change(channel, 123, 0.0);
        }
    }

    fn push(&mut self, packet: MidiPacket) {
        if let Some(producer) = self.producer.as_mut() {
            let _ = producer.try_push(packet);
        }
    }
}

impl Drop for LogicMidiOutput {
    fn drop(&mut self) {
        self.all_notes_off();
        self.stop.store(true, Ordering::Release);
    }
}

fn is_logic_audio_unit_host() -> bool {
    let host_is_logic = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .is_some_and(|name| {
            name.contains("AUHostingService")
                || name.contains("Logic Pro")
                || name.contains("GarageBand")
        });

    host_is_logic && loaded_from_guitar_component()
}

pub(crate) fn loaded_from_guitar_component() -> bool {
    let mut info = std::mem::MaybeUninit::<libc::Dl_info>::zeroed();
    // SAFETY: dladdr only inspects the image containing this function pointer
    // and initializes `info` when it succeeds.
    let found = unsafe {
        libc::dladdr(
            loaded_from_guitar_component as *const () as *const c_void,
            info.as_mut_ptr(),
        )
    };
    if found == 0 {
        return false;
    }
    let info = unsafe { info.assume_init() };
    if info.dli_fname.is_null() {
        return false;
    }
    // SAFETY: successful dladdr returns a NUL-terminated image path.
    let path = unsafe { CStr::from_ptr(info.dli_fname) };
    path.to_string_lossy()
        .contains("Contrapunk Guitar.component")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_channel_voice_messages() {
        assert_eq!(MidiPacket::note_on(2, 60, 1.0).as_slice(), &[0x92, 60, 127]);
        assert_eq!(MidiPacket::note_off(2, 60, 0.0).as_slice(), &[0x82, 60, 0]);
        assert_eq!(MidiPacket::pitch_bend(0, 0.5).as_slice(), &[0xE0, 0, 64]);
        assert_eq!(
            MidiPacket::channel_pressure(3, 1.0).as_slice(),
            &[0xD3, 127]
        );
    }
}
