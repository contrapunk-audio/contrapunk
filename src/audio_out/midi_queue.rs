//! Lock-free SPSC MIDI event queue between the harmony router and the
//! audio callback.

use ringbuf::{
    traits::{Consumer as _, Producer as _, Split as _},
    HeapRb,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MidiEvent {
    NoteOn { voice: u8, note: u8, velocity: u8 },
    NoteOff { voice: u8, note: u8 },
}

pub struct MidiProducer(ringbuf::HeapProd<MidiEvent>);
pub struct MidiConsumer(ringbuf::HeapCons<MidiEvent>);

#[derive(Debug, PartialEq, Eq)]
pub struct QueueFull;

impl MidiProducer {
    pub fn push(&mut self, event: MidiEvent) -> Result<(), QueueFull> {
        self.0.try_push(event).map_err(|_| QueueFull)
    }
}

impl MidiConsumer {
    pub fn pop(&mut self) -> Option<MidiEvent> {
        self.0.try_pop()
    }
}

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
