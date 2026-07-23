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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_fx_controls_cannot_poison_audio() {
        let mut drive = Drive::new();
        drive.set_drive(f32::NAN);
        drive.set_mix(f32::INFINITY);
        drive.drive = f32::NAN;
        drive.mix = f32::INFINITY;

        let mut delay = Delay::new(1024);
        delay.set_delay_secs(f32::NAN, 48_000.0);
        delay.set_feedback(f32::INFINITY);
        delay.set_mix(f32::NAN);

        let mut reverb = Reverb::new(f32::NAN);
        reverb.set_decay(f32::NAN);
        reverb.set_damping(f32::INFINITY);
        reverb.set_mix(f32::NEG_INFINITY);

        let mut fdn = FdnReverb::new(f32::NAN);
        fdn.set_decay_seconds(f32::NAN);
        fdn.set_damping(f32::INFINITY);
        fdn.set_mix(f32::NEG_INFINITY);

        let mut chorus = Chorus::new(f32::NAN);
        chorus.set_rate_hz(f32::NAN);
        chorus.set_depth_ms(f32::INFINITY);
        chorus.set_mix(f32::NEG_INFINITY);

        let mut flanger = Flanger::new(f32::NAN);
        flanger.set_rate_hz(f32::NAN);
        flanger.set_depth_ms(f32::INFINITY);
        flanger.set_feedback(f32::NEG_INFINITY);
        flanger.set_mix(f32::NAN);

        let mut phaser = Phaser::new(f32::NAN);
        phaser.set_rate_hz(f32::NAN);
        phaser.set_depth(f32::INFINITY);
        phaser.set_feedback(f32::NEG_INFINITY);
        phaser.set_mix(f32::NAN);

        let mut compressor = Compressor::new(f32::NAN);
        compressor.set_threshold_db(f32::NAN);
        compressor.set_ratio(f32::INFINITY);
        compressor.set_attack_ms(f32::NEG_INFINITY);
        compressor.set_release_ms(f32::NAN);
        compressor.set_makeup_db(f32::INFINITY);
        compressor.set_mix(f32::NEG_INFINITY);

        let mut slots = [
            FxSlot::Drive(drive),
            FxSlot::Delay(delay),
            FxSlot::Reverb(reverb),
            FxSlot::FdnReverb(fdn),
            FxSlot::Chorus(chorus),
            FxSlot::Flanger(flanger),
            FxSlot::Phaser(phaser),
            FxSlot::Compressor(compressor),
        ];
        for slot in &mut slots {
            let mut audio = [0.1; 64];
            slot.process_inplace(&mut audio, 2);
            assert!(
                audio.iter().all(|sample| sample.is_finite()),
                "{}",
                slot.name()
            );
        }
    }
}
