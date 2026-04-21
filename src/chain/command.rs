//! Commands for mutating a live audio chain from a non-audio thread.
//!
//! Non-audio threads (Tauri command handlers, UI) push `ChainCommand`
//! variants onto the SPSC queue owned by a [`crate::chain::ChainCommander`].
//! The audio thread drains the queue at the top of each
//! [`crate::chain::Chain::process`] call — so mutations take effect on
//! the next audio buffer without any locking in the hot path.
//!
//! Only [`ChainCommand::PushBlock`] transfers ownership of a boxed
//! block across threads; receive-side consumption simply moves it into
//! the chain's internal `Vec`. `Box<dyn AudioBlock>` is `Send` because
//! [`AudioBlock`][crate::chain::AudioBlock] requires `Send`.

use crate::chain::AudioBlock;

/// Mutation command for a live chain.
pub enum ChainCommand {
    /// Append a block at the end of the chain (output stage).
    PushBlock(Box<dyn AudioBlock>),
    /// Remove the block at the given index. Out-of-range removes are ignored.
    RemoveAt(usize),
    /// Drop every block in the chain.
    Clear,
}
