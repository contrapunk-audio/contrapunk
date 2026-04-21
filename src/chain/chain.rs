//! The [`Chain`] container — a linear pipeline of [`AudioBlock`]s.

use super::block::{AudioBlock, MidiBlockEvent};

/// A linear audio-processing chain.
///
/// Blocks are processed in order. The first block typically generates
/// audio (a synth); subsequent blocks shape it (FX). Every block
/// receives every MIDI event; FX blocks ignore them by default.
///
/// This v1 has no dynamic add/remove API — the chain is built once at
/// audio-clock startup. Rigs and plugin loading will introduce a
/// command queue later.
pub struct Chain {
    blocks: Vec<Box<dyn AudioBlock>>,
    sample_rate: u32,
}

impl Chain {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            blocks: Vec::new(),
            sample_rate,
        }
    }

    /// Append a block to the chain. The block's `set_sample_rate` is
    /// called so its DSP state is correct before the first `process`.
    pub fn push(&mut self, mut block: Box<dyn AudioBlock>) {
        block.set_sample_rate(self.sample_rate);
        self.blocks.push(block);
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

    /// Process one buffer. Each block processes in order on the same
    /// interleaved buffer. Interleaved channels ({@code channels} must
    /// match the cpal stream's channel count).
    pub fn process(&mut self, buffer: &mut [f32], channels: usize) {
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
}
