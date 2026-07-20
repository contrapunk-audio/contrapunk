//! Audio FX bus (Phase 21.A5 → A6).
//!
//! A5 introduced the slot-based Drive / Delay / Reverb bus. The current
//! A6 implementation exposes an 8-slot chain with the modulation and
//! dynamics family needed by the public surface: FDN-16 reverb, chorus,
//! flanger, phaser, and compressor. Effects allocate their delay/state
//! memory at construction and process in-place during the audio callback.

pub mod delay;
pub mod drive;
pub mod modulated;
pub mod reverb;

pub use delay::Delay;
pub use drive::Drive;
pub use modulated::{Chorus, Compressor, Flanger, Phaser};
pub use reverb::{FdnReverb, Reverb};

/// One slot of the FX chain. Empty slots are skipped during process.
pub enum FxSlot {
    Empty,
    Drive(Drive),
    Delay(Delay),
    Reverb(Reverb),
    FdnReverb(FdnReverb),
    Chorus(Chorus),
    Flanger(Flanger),
    Phaser(Phaser),
    Compressor(Compressor),
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
            FxSlot::FdnReverb(r) => r.process_inplace(buf, channels),
            FxSlot::Chorus(c) => c.process_inplace(buf, channels),
            FxSlot::Flanger(f) => f.process_inplace(buf, channels),
            FxSlot::Phaser(p) => p.process_inplace(buf, channels),
            FxSlot::Compressor(c) => c.process_inplace(buf, channels),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            FxSlot::Empty => "empty",
            FxSlot::Drive(_) => "drive",
            FxSlot::Delay(_) => "delay",
            FxSlot::Reverb(_) => "reverb",
            FxSlot::FdnReverb(_) => "fdn-reverb",
            FxSlot::Chorus(_) => "chorus",
            FxSlot::Flanger(_) => "flanger",
            FxSlot::Phaser(_) => "phaser",
            FxSlot::Compressor(_) => "compressor",
        }
    }
}

impl Default for FxSlot {
    fn default() -> Self {
        FxSlot::Empty
    }
}

/// Number of FX slots in the chain. A5 shipped four MVP slots; A6
/// expands this to the design-doc eight-slot reorderable chain surface.
pub const FX_SLOTS: usize = 8;
