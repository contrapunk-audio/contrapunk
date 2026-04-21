//! The [`Chain`] container — a linear pipeline of [`AudioBlock`]s.

use ringbuf::traits::Consumer;
use ringbuf::HeapCons;

use super::block::{AudioBlock, MidiBlockEvent};
use super::command::ChainCommand;

/// A linear audio-processing chain.
///
/// Blocks are processed in order. The first block typically generates
/// audio (a synth); subsequent blocks shape it (FX). Every block
/// receives every MIDI event; FX blocks ignore them by default.
///
/// Chains are mutated at runtime via a lock-free SPSC command queue
/// fed by a [`crate::chain::ChainCommander`]. The queue is drained at
/// the top of each [`Chain::process`] call so additions/removals take
/// effect on the next audio buffer without blocking the audio thread.
pub struct Chain {
    blocks: Vec<Box<dyn AudioBlock>>,
    sample_rate: u32,
    /// Consumer half of the command queue. `None` when the chain was
    /// built without a commander (tests, headless contexts).
    rx: Option<HeapCons<ChainCommand>>,
}

impl Chain {
    /// Create a chain with no command queue — useful for tests and
    /// static chains that never mutate at runtime.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            blocks: Vec::new(),
            sample_rate,
            rx: None,
        }
    }

    /// Create a chain wired to the consumer half of a command queue.
    /// Pair with [`crate::chain::ChainCommander::new_split`].
    pub fn with_queue(sample_rate: u32, rx: HeapCons<ChainCommand>) -> Self {
        Self {
            blocks: Vec::new(),
            sample_rate,
            rx: Some(rx),
        }
    }

    /// Append a block to the chain. The block's `set_sample_rate` is
    /// called so its DSP state is correct before the first `process`.
    pub fn push(&mut self, mut block: Box<dyn AudioBlock>) {
        block.set_sample_rate(self.sample_rate);
        self.blocks.push(block);
    }

    /// Drain pending [`ChainCommand`]s from the queue. Called at the
    /// top of `process()`; also safe to call from the audio thread
    /// directly (e.g. before the first buffer).
    fn drain_commands(&mut self) {
        let Some(rx) = self.rx.as_mut() else { return };
        while let Some(cmd) = rx.try_pop() {
            match cmd {
                ChainCommand::PushBlock(mut block) => {
                    block.set_sample_rate(self.sample_rate);
                    self.blocks.push(block);
                }
                ChainCommand::RemoveAt(idx) => {
                    if idx < self.blocks.len() {
                        self.blocks.remove(idx);
                    }
                }
                ChainCommand::Clear => {
                    self.blocks.clear();
                }
            }
        }
    }

    /// Reset all blocks. Safe to call from the audio thread.
    pub fn reset(&mut self) {
        for b in &mut self.blocks {
            b.reset();
        }
    }

    /// Deliver a MIDI event to every block. Synths act on it; FX
    /// ignore by default.
    pub fn midi_event(&mut self, event: MidiBlockEvent) {
        for b in &mut self.blocks {
            b.midi_event(event);
        }
    }

    /// Process one buffer. Drains pending `ChainCommand`s first, then
    /// runs each block in order on the same interleaved buffer.
    /// `channels` must match the cpal stream's channel count.
    pub fn process(&mut self, buffer: &mut [f32], channels: usize) {
        self.drain_commands();
        for b in &mut self.blocks {
            b.process(buffer, channels);
        }
    }

    /// Number of blocks currently in the chain.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// True if the chain has no blocks. Useful for safe no-op paths.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Update the sample rate on every block. Call when the device
    /// changes at runtime. Do NOT call mid-stream — blocks may have
    /// rate-dependent state that becomes invalid.
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        for b in &mut self.blocks {
            b.set_sample_rate(sample_rate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Silent;
    impl AudioBlock for Silent {
        fn name(&self) -> &str {
            "silent"
        }
        fn type_id(&self) -> &str {
            "test.silent"
        }
        fn process(&mut self, buffer: &mut [f32], _channels: usize) {
            for s in buffer.iter_mut() {
                *s = 0.0;
            }
        }
    }

    struct Gain {
        amount: f32,
    }
    impl AudioBlock for Gain {
        fn name(&self) -> &str {
            "gain"
        }
        fn type_id(&self) -> &str {
            "test.gain"
        }
        fn process(&mut self, buffer: &mut [f32], _channels: usize) {
            for s in buffer.iter_mut() {
                *s *= self.amount;
            }
        }
    }

    #[test]
    fn empty_chain_is_pass_through() {
        let mut chain = Chain::new(48_000);
        let mut buf = [0.5f32; 16];
        chain.process(&mut buf, 2);
        // Nothing ran; buffer unchanged.
        assert!(buf.iter().all(|&x| x == 0.5));
    }

    #[test]
    fn chain_runs_blocks_in_order() {
        let mut chain = Chain::new(48_000);
        chain.push(Box::new(Silent)); // zero the buffer
        chain.push(Box::new(Gain { amount: 2.0 })); // but 2x of zero is still zero
        let mut buf = [1.0f32; 16];
        chain.process(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn chain_gain_then_gain_composes() {
        let mut chain = Chain::new(48_000);
        chain.push(Box::new(Gain { amount: 3.0 }));
        chain.push(Box::new(Gain { amount: 0.5 }));
        let mut buf = [2.0f32; 8];
        chain.process(&mut buf, 2);
        // 2 * 3 * 0.5 = 3
        assert!(buf.iter().all(|&x| (x - 3.0).abs() < 1e-6));
    }

    #[test]
    fn len_and_empty() {
        let mut chain = Chain::new(48_000);
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
        chain.push(Box::new(Silent));
        assert!(!chain.is_empty());
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn queue_push_block_applies_on_next_process() {
        use crate::chain::ChainCommander;

        let (commander, rx) = ChainCommander::new_split();
        let mut chain = Chain::with_queue(48_000, rx);
        // Initial process: chain is empty, buffer passes through.
        let mut buf = [1.0f32; 8];
        chain.process(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 1.0));

        // Push a Silent block; next process should zero the buffer.
        commander.push_block(Box::new(Silent)).expect("push");
        chain.process(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn queue_remove_at_removes_block() {
        use crate::chain::ChainCommander;

        let (commander, rx) = ChainCommander::new_split();
        let mut chain = Chain::with_queue(48_000, rx);
        commander.push_block(Box::new(Silent)).expect("push");
        let mut buf = [1.0f32; 8];
        chain.process(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 0.0));

        commander.remove_at(0).expect("remove");
        // After removal the chain is empty again.
        for s in buf.iter_mut() {
            *s = 2.0;
        }
        chain.process(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 2.0));
    }

    #[test]
    fn queue_clear_empties_chain() {
        use crate::chain::ChainCommander;

        let (commander, rx) = ChainCommander::new_split();
        let mut chain = Chain::with_queue(48_000, rx);
        commander
            .push_block(Box::new(Gain { amount: 0.0 }))
            .expect("push");
        commander
            .push_block(Box::new(Gain { amount: 0.0 }))
            .expect("push");
        commander.clear().expect("clear");
        let mut buf = [0.5f32; 8];
        chain.process(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 0.5));
    }

    #[test]
    fn commander_mirror_tracks_pushes() {
        use crate::chain::ChainCommander;

        let (commander, _rx) = ChainCommander::new_split();
        commander.register_initial_blocks(vec![crate::chain::BlockDescriptor {
            type_id: "test.silent".into(),
            name: "silent".into(),
        }]);
        assert_eq!(commander.snapshot().len(), 1);
        commander
            .push_block(Box::new(Gain { amount: 1.0 }))
            .unwrap();
        let snap = commander.snapshot();
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[1].type_id, "test.gain");

        commander.remove_at(0).unwrap();
        assert_eq!(commander.snapshot().len(), 1);

        commander.clear().unwrap();
        assert_eq!(commander.snapshot().len(), 0);
    }
}
