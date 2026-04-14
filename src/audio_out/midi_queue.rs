//! Lock-free SPSC MIDI event queue between the harmony router and the
//! audio callback.
//!
//! The audio thread must never allocate or block. The harmony engine
//! (producer) and audio callback (consumer) communicate through a
//! bounded ringbuffer with static capacity.

use ringbuf::{
    traits::{Consumer as _, Producer as _, Split as _},
    HeapRb,
};

/// A MIDI event destined for the audio synth.
///
/// Voices are 0-indexed, matching Contrapunk's per-voice chain slots.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MidiEvent {
    NoteOn { voice: u8, note: u8, velocity: u8 },
    NoteOff { voice: u8, note: u8 },
}

/// Producer half of the MIDI queue. Held by the harmony router.
pub struct MidiProducer(ringbuf::HeapProd<MidiEvent>);

/// Consumer half of the MIDI queue. Held by the audio callback.
pub struct MidiConsumer(ringbuf::HeapCons<MidiEvent>);

/// Errors returned when pushing into a full MidiProducer.
#[derive(Debug, PartialEq, Eq)]
pub struct QueueFull;

impl MidiProducer {
    /// Push an event. Returns `Err(QueueFull)` if the queue is at capacity.
    /// The audio thread drains the queue each buffer, so `QueueFull` means
    /// something is very wrong (stalled audio thread or overflow attack).
    pub fn push(&mut self, event: MidiEvent) -> Result<(), QueueFull> {
        self.0.try_push(event).map_err(|_| QueueFull)
    }
}

impl MidiConsumer {
    /// Pop the next event. Returns `None` if the queue is empty.
    pub fn pop(&mut self) -> Option<MidiEvent> {
        self.0.try_pop()
    }
}

/// Create a new MIDI queue with the given capacity. Returns (producer, consumer).
///
/// The producer is held by the harmony router; the consumer is moved into
/// the audio callback. The capacity should be generous — at 48 kHz with
/// 256-sample buffers the audio thread runs ~188 times per second, so even
/// a bursty harmony engine rarely queues more than a handful of events
/// per buffer. Default callers use 1024.
pub fn midi_queue(capacity: usize) -> (MidiProducer, MidiConsumer) {
    let rb = HeapRb::<MidiEvent>::new(capacity);
    let (prod, cons) = rb.split();
    (MidiProducer(prod), MidiConsumer(cons))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop_note_on() {
        let (mut producer, mut consumer) = midi_queue(128);
        let evt = MidiEvent::NoteOn {
            voice: 0,
            note: 60,
            velocity: 100,
        };
        producer.push(evt).unwrap();
        assert_eq!(consumer.pop(), Some(evt));
        assert_eq!(consumer.pop(), None);
    }

    #[test]
    fn test_push_pop_note_off() {
        let (mut producer, mut consumer) = midi_queue(128);
        let evt = MidiEvent::NoteOff { voice: 2, note: 64 };
        producer.push(evt).unwrap();
        assert_eq!(consumer.pop(), Some(evt));
    }

    #[test]
    fn test_capacity_bound() {
        let (mut producer, _consumer) = midi_queue(2);
        producer
            .push(MidiEvent::NoteOn {
                voice: 0,
                note: 60,
                velocity: 100,
            })
            .unwrap();
        producer
            .push(MidiEvent::NoteOn {
                voice: 0,
                note: 61,
                velocity: 100,
            })
            .unwrap();
        let result = producer.push(MidiEvent::NoteOn {
            voice: 0,
            note: 62,
            velocity: 100,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_drain_into_vec() {
        let (mut producer, mut consumer) = midi_queue(128);
        producer
            .push(MidiEvent::NoteOn {
                voice: 0,
                note: 60,
                velocity: 100,
            })
            .unwrap();
        producer
            .push(MidiEvent::NoteOff { voice: 0, note: 60 })
            .unwrap();
        let mut out = Vec::with_capacity(2);
        while let Some(e) = consumer.pop() {
            out.push(e);
        }
        assert_eq!(out.len(), 2);
    }
}
