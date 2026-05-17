//! Audio FX bus (Phase 21.A5 v1).
//!
//! See `ELIXIR-DESIGN.md` §5 for the design-doc target (8-slot
//! reorderable chain, 2x oversampling around nonlinearities, 16-line
//! FDN reverb). A5 v1 ships three FX kinds (Drive / Delay / Reverb)
//! in a 4-slot chain; oversampling and EQ land in A5 follow-ups, the
//! FDN-16 reverb and the rest of the FX family land in A6.

pub mod delay;
pub mod drive;
pub mod reverb;

pub use delay::Delay;
pub use drive::Drive;
pub use reverb::Reverb;

/// One slot of the FX chain. Empty slots are skipped during process.
pub enum FxSlot {
    Empty,
    Drive(Drive),
    Delay(Delay),
    Reverb(Reverb),
}

impl FxSlot {
    pub const fn empty() -> Self {
        FxSlot::Empty
    }

    pub fn process_inplace(&mut self, buf: &mut [f32], channels: usize) {
        match self {
            FxSlot::Empty => {}
            FxSlot::Drive(d) => d.process_inplace(buf),
            FxSlot::Delay(d) => d.process_inplace(buf, channels),
            FxSlot::Reverb(r) => r.process_inplace(buf, channels),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            FxSlot::Empty => "empty",
            FxSlot::Drive(_) => "drive",
            FxSlot::Delay(_) => "delay",
            FxSlot::Reverb(_) => "reverb",
        }
    }
}

impl Default for FxSlot {
    fn default() -> Self {
        FxSlot::Empty
    }
}

/// Number of FX slots in the chain.
pub const FX_SLOTS: usize = 4;
