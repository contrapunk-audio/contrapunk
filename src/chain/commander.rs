//! Main-thread handle for mutating a live [`crate::chain::Chain`].
//!
//! The commander owns the producer half of a lock-free SPSC queue
//! feeding the audio thread, plus a main-thread mirror of block
//! descriptors so the UI can render the chain without probing the
//! audio thread.
//!
//! The mirror is updated synchronously whenever a command is pushed
//! — so from the UI's point of view the chain appears to mutate
//! immediately. The audio thread applies the real mutation on the
//! next buffer (sub-millisecond at typical block sizes).

use std::sync::Mutex;

use ringbuf::traits::{Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use serde::Serialize;

use super::block::AudioBlock;
use super::command::ChainCommand;

/// Metadata shown to the UI for a block in the chain.
#[derive(Clone, Debug, Serialize)]
pub struct BlockDescriptor {
    /// Stable type identifier (e.g. `"builtin.synth"`, `"clap:<path>"`).
    pub type_id: String,
    /// Human-readable name (e.g. `"Synth"`, plugin display name).
    pub name: String,
}

impl BlockDescriptor {
    pub fn from_block(b: &dyn AudioBlock) -> Self {
        Self {
            type_id: b.type_id().to_string(),
            name: b.name().to_string(),
        }
    }
}

/// Default capacity for the command queue. Commands are infrequent
/// (plugin add/remove from the UI) so a small capacity is fine; any
/// overflow is surfaced back to the caller via a `Result::Err`.
const DEFAULT_QUEUE_CAPACITY: usize = 64;

/// Main-thread handle for chain mutation. Clone-safe — internally
/// wraps a `Mutex` around the producer so multiple Tauri commands can
/// race to push commands without trampling each other.
pub struct ChainCommander {
    tx: Mutex<HeapProd<ChainCommand>>,
    /// Mirrored list of blocks. Updated in lockstep with pushes so the
    /// UI can render without probing the audio thread.
    mirror: Mutex<Vec<BlockDescriptor>>,
}

impl ChainCommander {
    /// Allocate a new SPSC queue and return (commander, consumer).
    /// Pass the consumer to [`crate::chain::Chain::with_queue`].
    pub fn new_split() -> (Self, HeapCons<ChainCommand>) {
        Self::new_split_with_capacity(DEFAULT_QUEUE_CAPACITY)
    }

    pub fn new_split_with_capacity(capacity: usize) -> (Self, HeapCons<ChainCommand>) {
        let rb = HeapRb::<ChainCommand>::new(capacity);
        let (tx, rx) = rb.split();
        (
            Self {
                tx: Mutex::new(tx),
                mirror: Mutex::new(Vec::new()),
            },
            rx,
        )
    }

    /// Record the descriptors for blocks pushed directly onto the
    /// chain at startup (before the commander was exposed to the UI).
    /// Keeps the mirror consistent with the chain's initial layout.
    pub fn register_initial_blocks(&self, descriptors: Vec<BlockDescriptor>) {
        let mut m = self.mirror.lock().expect("chain mirror mutex");
        *m = descriptors;
    }

    /// Push a block onto the chain. Returns the block back to the
    /// caller if the queue is full.
    pub fn push_block(&self, block: Box<dyn AudioBlock>) -> Result<BlockDescriptor, String> {
        let descriptor = BlockDescriptor::from_block(block.as_ref());
        let mut tx = self.tx.lock().map_err(|e| format!("commander lock: {e}"))?;
        if tx.try_push(ChainCommand::PushBlock(block)).is_err() {
            return Err("chain command queue full".into());
        }
        self.mirror
            .lock()
            .expect("chain mirror mutex")
            .push(descriptor.clone());
        Ok(descriptor)
    }

    /// Remove the block at `index`. No-op if out of range.
    pub fn remove_at(&self, index: usize) -> Result<(), String> {
        let mut tx = self.tx.lock().map_err(|e| format!("commander lock: {e}"))?;
        if tx.try_push(ChainCommand::RemoveAt(index)).is_err() {
            return Err("chain command queue full".into());
        }
        let mut m = self.mirror.lock().expect("chain mirror mutex");
        if index < m.len() {
            m.remove(index);
        }
        Ok(())
    }

    /// Drop every block. Useful for rig reload.
    pub fn clear(&self) -> Result<(), String> {
        let mut tx = self.tx.lock().map_err(|e| format!("commander lock: {e}"))?;
        if tx.try_push(ChainCommand::Clear).is_err() {
            return Err("chain command queue full".into());
        }
        self.mirror.lock().expect("chain mirror mutex").clear();
        Ok(())
    }

    /// Snapshot the main-thread mirror of block descriptors.
    pub fn snapshot(&self) -> Vec<BlockDescriptor> {
        self.mirror.lock().expect("chain mirror mutex").clone()
    }
}
