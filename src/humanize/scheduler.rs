use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::config::HumanizedNote;

/// A note scheduled to be sent at a specific time.
#[derive(Clone, Debug)]
struct ScheduledNote {
    send_at_ms: f64,
    note: HumanizedNote,
}

impl PartialEq for ScheduledNote {
    fn eq(&self, other: &Self) -> bool {
        self.send_at_ms == other.send_at_ms
    }
}

impl Eq for ScheduledNote {}

impl PartialOrd for ScheduledNote {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledNote {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering: smallest send_at_ms has highest priority
        other
            .send_at_ms
            .partial_cmp(&self.send_at_ms)
            .unwrap_or(Ordering::Equal)
    }
}

/// Priority queue for delayed humanized notes.
pub struct DelayQueue {
    heap: BinaryHeap<ScheduledNote>,
}

impl DelayQueue {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    /// Push a humanized note, scheduling it at `now_ms + note.delay_ms`.
    pub fn push(&mut self, note: HumanizedNote, now_ms: f64) {
        let send_at_ms = now_ms + note.delay_ms as f64;
        self.heap.push(ScheduledNote { send_at_ms, note });
    }

    /// Drain all notes whose scheduled time has arrived.
    pub fn drain_ready(&mut self, now_ms: f64) -> Vec<HumanizedNote> {
        let mut ready = Vec::new();
        while let Some(top) = self.heap.peek() {
            if top.send_at_ms <= now_ms {
                ready.push(self.heap.pop().unwrap().note);
            } else {
                break;
            }
        }
        ready
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
