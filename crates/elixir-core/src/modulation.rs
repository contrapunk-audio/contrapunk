//! Modulation matrix (Phase 21.A3 v1).
//!
//! Sparse list of `(source, destination, amount)` routes evaluated once
//! per processing block. Sources produce a value in roughly `[-1, 1]`;
//! destinations accumulate (`amount * src`) into the engine's per-block
//! mod buffers; the engine reads those buffers when computing master
//! gain / LFO rate / oscillator pitch etc.
//!
//! Design follows `ELIXIR-DESIGN.md` §6: SoA storage with typed IDs.
//! A3 v1 simplifies the design-doc spec by:
//!
//! - Skipping `power` curve shaping (linear amounts only — A6 brings
//!   the power-curve back as part of the multi-stage envelope work)
//! - Skipping per-route line-mapping curves (same A6 deferral)
//! - Treating every source as global; per-voice sources (per-voice
//!   envelopes) feed the matrix via their averaged value across active
//!   voices. Proper per-voice routing is an A3 follow-up.
//! - No `arc-swap` table hot-swap yet; the route table is mutable
//!   directly on the engine. Lock-free UI→audio routing lands in B4.

/// Where a modulation value comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModSrc {
    /// Constant 1.0 — useful for a manual offset on any destination.
    Constant,
    /// Global LFO `[0, MAX_GLOBAL_LFOS)`.
    Lfo(u8),
    /// Global average of the per-voice amp envelope across all active
    /// voices. Coarse, but enough for mod-of-mod demos at A3.
    AmpEnv,
}

/// What a modulation value flows into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModDest {
    /// Adds to master output gain. Bipolar source values produce
    /// classic tremolo when set to a small amount.
    MasterGain,
    /// Adds to global LFO `i` rate in Hz.
    LfoRate(u8),
}

/// One routing entry. `amount` is the multiplier applied to the source
/// value before summation at the destination. `bipolar` is informational
/// for A3; it'll gate the `-0.5` re-centering in a future refinement.
#[derive(Clone, Copy, Debug)]
pub struct ModRoute {
    pub src: ModSrc,
    pub dst: ModDest,
    pub amount: f32,
    pub bipolar: bool,
}

impl ModRoute {
    pub const fn new(src: ModSrc, dst: ModDest, amount: f32) -> Self {
        Self {
            src,
            dst,
            amount,
            bipolar: true,
        }
    }
}

/// Maximum simultaneous modulation routes. The design-doc target is
/// 64; A3 v1 caps lower to keep the per-block iteration cheap.
pub const MAX_ROUTES: usize = 32;

/// Maximum global LFOs. A3 v1 ships with one but the array is sized
/// for design-doc parity.
pub const MAX_GLOBAL_LFOS: usize = 4;

/// Aggregated per-destination mod values computed each block, plus the
/// route table itself.
pub struct ModMatrix {
    routes: [Option<ModRoute>; MAX_ROUTES],
    next_slot: usize,
    /// Last-evaluated destination buffers. Indexed by ModDest variant.
    pub master_gain_mod: f32,
    pub lfo_rate_mod_hz: [f32; MAX_GLOBAL_LFOS],
}

impl ModMatrix {
    pub const fn new() -> Self {
        Self {
            routes: [None; MAX_ROUTES],
            next_slot: 0,
            master_gain_mod: 0.0,
            lfo_rate_mod_hz: [0.0; MAX_GLOBAL_LFOS],
        }
    }

    /// Add a route. Returns the slot index, or `None` if full.
    pub fn add_route(&mut self, route: ModRoute) -> Option<usize> {
        for i in self.next_slot..MAX_ROUTES {
            if self.routes[i].is_none() {
                self.routes[i] = Some(route);
                self.next_slot = i + 1;
                return Some(i);
            }
        }
        for (i, slot) in self.routes.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(route);
                self.next_slot = i + 1;
                return Some(i);
            }
        }
        None
    }

    pub fn remove_route(&mut self, idx: usize) {
        if idx < MAX_ROUTES {
            self.routes[idx] = None;
            if idx < self.next_slot {
                self.next_slot = idx;
            }
        }
    }

    pub fn clear(&mut self) {
        for slot in self.routes.iter_mut() {
            *slot = None;
        }
        self.next_slot = 0;
        self.master_gain_mod = 0.0;
        self.lfo_rate_mod_hz = [0.0; MAX_GLOBAL_LFOS];
    }

    pub fn route_count(&self) -> usize {
        self.routes.iter().filter(|r| r.is_some()).count()
    }

    /// Reset destination accumulators. Called by the engine at the
    /// start of each block.
    pub fn reset_destinations(&mut self) {
        self.master_gain_mod = 0.0;
        self.lfo_rate_mod_hz = [0.0; MAX_GLOBAL_LFOS];
    }

    /// Iterate sources in the order the engine needs them. The engine
    /// supplies a callback that returns the source value; this struct
    /// then routes that into the destination accumulator.
    pub fn route_for_source<F: FnMut(ModSrc) -> f32>(&mut self, mut value_of: F) {
        for route in self.routes.iter().flatten() {
            let v = value_of(route.src) * route.amount;
            match route.dst {
                ModDest::MasterGain => self.master_gain_mod += v,
                ModDest::LfoRate(i) => {
                    let idx = i as usize;
                    if idx < MAX_GLOBAL_LFOS {
                        self.lfo_rate_mod_hz[idx] += v;
                    }
                }
            }
        }
    }
}

impl Default for ModMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// One-pole linear smoother for control-rate parameters. Click-free
/// `set_target` → `tick` transition over a configurable half-life.
#[derive(Clone, Copy, Debug)]
pub struct Smoothed {
    current: f32,
    target: f32,
    coeff: f32,
}

impl Smoothed {
    pub const fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            coeff: 1.0,
        }
    }
    /// Coefficient is `1 - exp(-block / (sr * half_life))` per call to
    /// [`Smoothed::step`]. Compute once when sample rate / block size
    /// is known.
    pub fn set_coeff(&mut self, coeff: f32) {
        self.coeff = coeff.clamp(0.0, 1.0);
    }
    pub fn set(&mut self, value: f32) {
        self.target = value;
    }
    pub fn snap_to(&mut self, value: f32) {
        self.current = value;
        self.target = value;
    }
    pub fn value(&self) -> f32 {
        self.current
    }
    pub fn step(&mut self) -> f32 {
        self.current += (self.target - self.current) * self.coeff;
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_matrix_has_zero_destinations() {
        let mut m = ModMatrix::new();
        m.route_for_source(|_| 1.0);
        assert_eq!(m.master_gain_mod, 0.0);
        assert_eq!(m.lfo_rate_mod_hz[0], 0.0);
    }

    #[test]
    fn single_route_routes_value() {
        let mut m = ModMatrix::new();
        m.add_route(ModRoute::new(ModSrc::Constant, ModDest::MasterGain, 0.5))
            .unwrap();
        m.route_for_source(|src| match src {
            ModSrc::Constant => 1.0,
            _ => 0.0,
        });
        assert!((m.master_gain_mod - 0.5).abs() < 1e-6);
    }

    #[test]
    fn multiple_routes_sum() {
        let mut m = ModMatrix::new();
        m.add_route(ModRoute::new(ModSrc::Constant, ModDest::MasterGain, 0.25))
            .unwrap();
        m.add_route(ModRoute::new(ModSrc::Lfo(0), ModDest::MasterGain, 0.1))
            .unwrap();
        m.route_for_source(|src| match src {
            ModSrc::Constant => 1.0,
            ModSrc::Lfo(0) => 0.5,
            _ => 0.0,
        });
        // 0.25 * 1.0 + 0.1 * 0.5 = 0.30
        assert!((m.master_gain_mod - 0.30).abs() < 1e-6);
    }

    #[test]
    fn lfo_rate_dest_routes_to_correct_lfo_index() {
        let mut m = ModMatrix::new();
        m.add_route(ModRoute::new(ModSrc::AmpEnv, ModDest::LfoRate(1), 3.0))
            .unwrap();
        m.route_for_source(|src| match src {
            ModSrc::AmpEnv => 1.0,
            _ => 0.0,
        });
        assert_eq!(m.lfo_rate_mod_hz[0], 0.0);
        assert!((m.lfo_rate_mod_hz[1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn smoothed_ramps_toward_target() {
        let mut s = Smoothed::new(0.0);
        s.set_coeff(0.5);
        s.set(1.0);
        let v1 = s.step();
        let v2 = s.step();
        let v3 = s.step();
        assert!(v1 > 0.0 && v1 < 1.0);
        assert!(v3 > v2 && v2 > v1);
        assert!(v3 < 1.0);
    }

    #[test]
    fn remove_and_add_route() {
        let mut m = ModMatrix::new();
        let i = m
            .add_route(ModRoute::new(ModSrc::Constant, ModDest::MasterGain, 1.0))
            .unwrap();
        assert_eq!(m.route_count(), 1);
        m.remove_route(i);
        assert_eq!(m.route_count(), 0);
        m.add_route(ModRoute::new(ModSrc::Constant, ModDest::MasterGain, 0.5))
            .unwrap();
        assert_eq!(m.route_count(), 1);
    }
}
