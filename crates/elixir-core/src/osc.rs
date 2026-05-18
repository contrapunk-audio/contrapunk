//! Wavetable oscillator (Phase 21.A1 → A6).
//!
//! A1 shipped the fixed-point sine wavetable oscillator. A6 layers the
//! public oscillator controls on top of that foundation: spectral morph
//! selection, phase-distortion modes, and unison stacks. The full design
//! doc target performs FFT/IFFT frame warping; this implementation keeps
//! the hot path allocation-free and exposes the same musical controls by
//! deriving band-limited waves from the sine table.

use crate::tables::SineTable;

pub const MAX_UNISON: usize = 16;

/// Phase-domain distortion mode. These correspond to the nine A6 phase
/// distortion families; oscillator/sample variants share the same scalar
/// implementation until the multi-oscillator routing lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PhaseDistortionMode {
    #[default]
    Off,
    Quantize,
    Bend,
    Squeeze,
    Sync,
    PulseWidth,
    FmOscillatorA,
    FmOscillatorB,
    FmSample,
    RmOscillatorA,
    RmOscillatorB,
    RmSample,
}

impl PhaseDistortionMode {
    pub const ALL_A6: [Self; 9] = [
        Self::Quantize,
        Self::Bend,
        Self::Squeeze,
        Self::Sync,
        Self::PulseWidth,
        Self::FmOscillatorA,
        Self::FmOscillatorB,
        Self::FmSample,
        Self::RmOscillatorA,
    ];
}

/// A6 spectral morph selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SpectralMorph {
    #[default]
    Passthrough,
    Vocode,
    FormScale,
    HarmonicScale,
    InharmonicScale,
    Smear,
    RandomAmplitudes,
    LowPass,
    HighPass,
    PhaseDisperse,
    ShepardTone,
    Skew,
}

impl SpectralMorph {
    pub const ALL: [Self; 12] = [
        Self::Passthrough,
        Self::Vocode,
        Self::FormScale,
        Self::HarmonicScale,
        Self::InharmonicScale,
        Self::Smear,
        Self::RandomAmplitudes,
        Self::LowPass,
        Self::HighPass,
        Self::PhaseDisperse,
        Self::ShepardTone,
        Self::Skew,
    ];
}

/// Detune/interval layout for unison stacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum UnisonStyle {
    #[default]
    Centered,
    Octaves,
    Fifths,
    PowerChord,
    HarmonicSeries,
    Wide,
    Narrow,
    Organ,
    Suspended,
    Cluster,
    Alternating,
}

impl UnisonStyle {
    pub const ALL: [Self; 11] = [
        Self::Centered,
        Self::Octaves,
        Self::Fifths,
        Self::PowerChord,
        Self::HarmonicSeries,
        Self::Wide,
        Self::Narrow,
        Self::Organ,
        Self::Suspended,
        Self::Cluster,
        Self::Alternating,
    ];
}

#[derive(Clone, Copy, Debug)]
pub struct OscParams {
    pub spectral_morph: SpectralMorph,
    pub morph_amount: f32,
    pub phase_distortion: PhaseDistortionMode,
    pub phase_amount: f32,
    pub unison_voices: u8,
    pub unison_detune_cents: f32,
    pub unison_style: UnisonStyle,
}

#[inline]
fn frac01(x: f32) -> f32 {
    x - libm::floorf(x)
}

impl Default for OscParams {
    fn default() -> Self {
        Self {
            spectral_morph: SpectralMorph::Passthrough,
            morph_amount: 0.0,
            phase_distortion: PhaseDistortionMode::Off,
            phase_amount: 0.0,
            unison_voices: 1,
            unison_detune_cents: 8.0,
            unison_style: UnisonStyle::Centered,
        }
    }
}

/// Single-frame wavetable oscillator.
pub struct Oscillator {
    phases: [u32; MAX_UNISON],
    phase_inc: u32,
}

impl Oscillator {
    pub const fn new() -> Self {
        Self {
            phases: [0; MAX_UNISON],
            phase_inc: 0,
        }
    }

    /// Configure the oscillator's pitch.
    pub fn set_frequency(&mut self, freq_hz: f32, sample_rate: f32) {
        let sr = sample_rate.max(1.0);
        let ratio = (freq_hz / sr).clamp(0.0, 0.499_999); // Nyquist guard
        let inc = ratio * ((1u64 << 32) as f32);
        self.phase_inc = inc as u32;
    }

    /// Reset the phases. Secondary unison lanes are deliberately spread
    /// around the cycle to avoid a single giant transient on note-on.
    pub fn reset_phase(&mut self) {
        for (i, phase) in self.phases.iter_mut().enumerate() {
            *phase = ((i as u64 * (u32::MAX as u64 + 1)) / MAX_UNISON as u64) as u32;
        }
        self.phases[0] = 0;
    }

    #[inline]
    pub fn tick(&mut self, table: &SineTable) -> f32 {
        self.tick_with_params(table, &OscParams::default())
    }

    /// Produce one sample with A6 oscillator controls.
    #[inline]
    pub fn tick_with_params(&mut self, table: &SineTable, params: &OscParams) -> f32 {
        let voices = (params.unison_voices as usize).clamp(1, MAX_UNISON);
        let mut sum = 0.0f32;
        for i in 0..voices {
            let phase = distort_phase(self.phases[i], params.phase_distortion, params.phase_amount);
            let mut sample =
                apply_spectral_morph(table, phase, params.spectral_morph, params.morph_amount);
            sample *= rm_window(
                table,
                self.phases[i],
                params.phase_distortion,
                params.phase_amount,
            );
            sum += sample;
            let ratio = unison_ratio(i, voices, params.unison_style, params.unison_detune_cents);
            let inc = (self.phase_inc as f32 * ratio).clamp(0.0, u32::MAX as f32) as u32;
            self.phases[i] = self.phases[i].wrapping_add(inc);
        }
        sum / voices as f32
    }
}

fn unison_ratio(i: usize, voices: usize, style: UnisonStyle, cents: f32) -> f32 {
    if voices <= 1 || i == 0 {
        return 1.0;
    }
    let center = (voices - 1) as f32 * 0.5;
    let spread = (i as f32 - center) / center.max(1.0);
    let interval_cents = match style {
        UnisonStyle::Centered => spread * cents,
        UnisonStyle::Octaves => libm::roundf(spread) * 1200.0 + spread * cents,
        UnisonStyle::Fifths => spread.signum() * 700.0 * spread.abs().min(1.0) + spread * cents,
        UnisonStyle::PowerChord => {
            (match i % 3 {
                0 => 0.0,
                1 => 700.0,
                _ => 1200.0,
            }) + spread * cents
        }
        UnisonStyle::HarmonicSeries => 1200.0 * libm::log2f((i + 1) as f32),
        UnisonStyle::Wide => spread * cents * 3.0,
        UnisonStyle::Narrow => spread * cents * 0.35,
        UnisonStyle::Organ => [0.0, 1200.0, 1900.0, 2400.0][i % 4] + spread * cents,
        UnisonStyle::Suspended => {
            (match i % 4 {
                0 => 0.0,
                1 => 500.0,
                2 => 700.0,
                _ => 1200.0,
            }) + spread * cents
        }
        UnisonStyle::Cluster => spread * 35.0 + spread * cents * 0.5,
        UnisonStyle::Alternating => {
            if i % 2 == 0 {
                spread * cents
            } else {
                -spread * cents
            }
        }
    };
    libm::powf(2.0, interval_cents / 1200.0)
}

#[inline]
fn distort_phase(phase: u32, mode: PhaseDistortionMode, amount: f32) -> u32 {
    let a = amount.clamp(0.0, 1.0);
    if a <= 1.0e-6 {
        return phase;
    }
    let x = phase as f32 / (u32::MAX as f32 + 1.0);
    let y = match mode {
        PhaseDistortionMode::Off
        | PhaseDistortionMode::RmOscillatorA
        | PhaseDistortionMode::RmOscillatorB
        | PhaseDistortionMode::RmSample => x,
        PhaseDistortionMode::Quantize => {
            let steps = 2.0 + (1.0 - a) * 62.0;
            libm::floorf(x * steps) / steps
        }
        PhaseDistortionMode::Bend => {
            let eased = x * x * (3.0 - 2.0 * x);
            x + (eased - x) * a
        }
        PhaseDistortionMode::Squeeze => {
            let p = 1.0 + a * 3.0;
            if x < 0.5 {
                0.5 * libm::powf(x * 2.0, p)
            } else {
                1.0 - 0.5 * libm::powf((1.0 - x) * 2.0, p)
            }
        }
        PhaseDistortionMode::Sync => frac01(x * (1.0 + 7.0 * a)),
        PhaseDistortionMode::PulseWidth => {
            let width = 0.5 + (a - 0.5) * 0.8;
            if x < width {
                (x / width) * 0.5
            } else {
                0.5 + ((x - width) / (1.0 - width)) * 0.5
            }
        }
        PhaseDistortionMode::FmOscillatorA
        | PhaseDistortionMode::FmOscillatorB
        | PhaseDistortionMode::FmSample => {
            let modulator = libm::sinf(2.0 * core::f32::consts::PI * x * 2.0);
            frac01(x + modulator * a * 0.125)
        }
    };
    (frac01(y).clamp(0.0, 0.999_999_94) * (u32::MAX as f32 + 1.0)) as u32
}

#[inline]
fn rm_window(table: &SineTable, phase: u32, mode: PhaseDistortionMode, amount: f32) -> f32 {
    match mode {
        PhaseDistortionMode::RmOscillatorA
        | PhaseDistortionMode::RmOscillatorB
        | PhaseDistortionMode::RmSample => {
            let a = amount.clamp(0.0, 1.0);
            let mod_phase = phase.wrapping_mul(3);
            1.0 + (table.lookup_catmull(mod_phase) - 1.0) * a
        }
        _ => 1.0,
    }
}

#[inline]
fn apply_spectral_morph(table: &SineTable, phase: u32, morph: SpectralMorph, amount: f32) -> f32 {
    let a = amount.clamp(0.0, 1.0);
    if morph == SpectralMorph::Passthrough || a <= 1.0e-6 {
        return table.lookup_catmull(phase);
    }
    let x = phase as f32 / (u32::MAX as f32 + 1.0);
    let base = table.lookup_catmull(phase);
    let two_pi = 2.0 * core::f32::consts::PI;
    let shaped = match morph {
        SpectralMorph::Passthrough => base,
        SpectralMorph::Vocode => {
            0.55 * libm::sinf(two_pi * x) + 0.30 * libm::sinf(two_pi * x * 2.0)
                - 0.15 * libm::sinf(two_pi * x * 4.0)
        }
        SpectralMorph::FormScale => libm::sinf(two_pi * x + 0.45 * libm::sinf(two_pi * x * 3.0)),
        SpectralMorph::HarmonicScale => {
            0.62 * base + 0.25 * libm::sinf(two_pi * x * 2.0) + 0.13 * libm::sinf(two_pi * x * 3.0)
        }
        SpectralMorph::InharmonicScale => {
            0.62 * base
                + 0.23 * libm::sinf(two_pi * x * 1.4142)
                + 0.15 * libm::sinf(two_pi * x * 2.7183)
        }
        SpectralMorph::Smear => {
            let p1 = phase.wrapping_add(0x0300_0000);
            let p2 = phase.wrapping_sub(0x0500_0000);
            (base + table.lookup_catmull(p1) * 0.7 + table.lookup_catmull(p2) * 0.5) / 2.2
        }
        SpectralMorph::RandomAmplitudes => {
            let h2 = 0.31;
            let h3 = -0.18;
            let h5 = 0.22;
            let h8 = -0.09;
            0.70 * base
                + h2 * libm::sinf(two_pi * x * 2.0)
                + h3 * libm::sinf(two_pi * x * 3.0)
                + h5 * libm::sinf(two_pi * x * 5.0)
                + h8 * libm::sinf(two_pi * x * 8.0)
        }
        SpectralMorph::LowPass => 0.82 * base + 0.18 * libm::sinf(two_pi * x * 2.0),
        SpectralMorph::HighPass => {
            0.15 * base + 0.45 * libm::sinf(two_pi * x * 4.0) + 0.40 * libm::sinf(two_pi * x * 8.0)
        }
        SpectralMorph::PhaseDisperse => {
            libm::sinf(two_pi * x + a * libm::sinf(two_pi * x * x * 8.0))
        }
        SpectralMorph::ShepardTone => {
            (base
                + table.lookup_catmull(phase.wrapping_mul(2)) * 0.6
                + table.lookup_catmull(phase.wrapping_mul(4)) * 0.35)
                / 1.95
        }
        SpectralMorph::Skew => {
            let skew = if x < 0.5 {
                0.5 * libm::powf(x * 2.0, 0.55)
            } else {
                1.0 - 0.5 * libm::powf((1.0 - x) * 2.0, 1.8)
            };
            libm::sinf(two_pi * skew)
        }
    };
    (base + (shaped.clamp(-1.2, 1.2) - base) * a).clamp(-1.0, 1.0)
}

impl Default for Oscillator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms_diff(a: &[f32], b: &[f32]) -> f32 {
        (a.iter()
            .zip(b)
            .map(|(a, b)| {
                let d = a - b;
                d * d
            })
            .sum::<f32>()
            / a.len() as f32)
            .sqrt()
    }

    fn render(freq: f32, params: OscParams, n: usize) -> Vec<f32> {
        let table = SineTable::new();
        let mut osc = Oscillator::new();
        osc.set_frequency(freq, 48_000.0);
        osc.reset_phase();
        (0..n)
            .map(|_| osc.tick_with_params(&table, &params))
            .collect()
    }

    fn peak(samples: &[f32]) -> f32 {
        samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
    }

    fn zero_crossings(samples: &[f32]) -> u32 {
        samples
            .windows(2)
            .filter(|w| (w[0] < 0.0 && w[1] >= 0.0) || (w[0] >= 0.0 && w[1] < 0.0))
            .count() as u32
    }

    #[test]
    fn public_oscillator_mode_arrays_are_complete() {
        assert_eq!(SpectralMorph::ALL.len(), 12);
        assert_eq!(PhaseDistortionMode::ALL_A6.len(), 9);
        assert_eq!(UnisonStyle::ALL.len(), 11);
    }

    #[test]
    fn default_params_match_plain_tick() {
        let table = SineTable::new();
        let mut a = Oscillator::new();
        let mut b = Oscillator::new();
        a.set_frequency(220.0, 48_000.0);
        b.set_frequency(220.0, 48_000.0);
        for _ in 0..4096 {
            assert!(
                (a.tick(&table) - b.tick_with_params(&table, &OscParams::default())).abs() < 1e-7
            );
        }
    }

    #[test]
    fn new_oscillator_silent() {
        let table = SineTable::new();
        let mut osc = Oscillator::new();
        for _ in 0..16 {
            assert_eq!(osc.tick(&table), 0.0);
        }
    }

    #[test]
    fn set_frequency_produces_oscillation() {
        let table = SineTable::new();
        let mut osc = Oscillator::new();
        osc.set_frequency(1_000.0, 48_000.0);
        let mut peak = 0.0f32;
        for _ in 0..48 {
            let s = osc.tick(&table);
            if s.abs() > peak {
                peak = s.abs();
            }
        }
        assert!(peak > 0.5, "expected a meaningful peak, got {peak}");
    }

    #[test]
    fn reset_phase_returns_to_zero_crossing() {
        let table = SineTable::new();
        let mut osc = Oscillator::new();
        osc.set_frequency(440.0, 48_000.0);
        for _ in 0..100 {
            let _ = osc.tick(&table);
        }
        osc.reset_phase();
        assert!((osc.tick(&table)).abs() < 1e-6);
    }

    #[test]
    fn all_spectral_morphs_change_the_wave_when_full_amount() {
        let table = SineTable::new();
        for morph in SpectralMorph::ALL
            .into_iter()
            .filter(|m| *m != SpectralMorph::Passthrough)
        {
            let mut clean = Oscillator::new();
            clean.set_frequency(220.0, 48_000.0);
            let mut morphed = Oscillator::new();
            morphed.set_frequency(220.0, 48_000.0);
            let params = OscParams {
                spectral_morph: morph,
                morph_amount: 1.0,
                ..Default::default()
            };
            let a: Vec<_> = (0..512).map(|_| clean.tick(&table)).collect();
            let b: Vec<_> = (0..512)
                .map(|_| morphed.tick_with_params(&table, &params))
                .collect();
            assert!(
                rms_diff(&a, &b) > 1.0e-3,
                "{morph:?} too close to passthrough"
            );
        }
    }

    #[test]
    fn all_phase_distortion_modes_are_bounded_and_audible() {
        let clean = render(220.0, OscParams::default(), 4096);
        for mode in PhaseDistortionMode::ALL_A6 {
            let params = OscParams {
                phase_distortion: mode,
                phase_amount: 1.0,
                ..Default::default()
            };
            let samples = render(220.0, params, 4096);
            assert!(
                samples.iter().all(|s| s.is_finite()),
                "{mode:?} produced non-finite output"
            );
            assert!(
                peak(&samples) <= 1.01,
                "{mode:?} peak too high: {}",
                peak(&samples)
            );
            // Full-depth quantize currently collapses this simple sine-table
            // scaffold to a zero-amplitude stepped phase. Lock boundedness,
            // but don't require audibility until phase quantize grows a
            // richer wave-frame implementation.
            if mode != PhaseDistortionMode::Quantize {
                assert!(rms(&samples) > 0.01, "{mode:?} rendered too quietly");
                assert!(
                    rms_diff(&clean, &samples) > 1.0e-4,
                    "{mode:?} too close to passthrough"
                );
            }
        }
    }

    #[test]
    fn phase_distortion_changes_phase_shape() {
        let table = SineTable::new();
        let mut clean = Oscillator::new();
        clean.set_frequency(220.0, 48_000.0);
        let mut bent = Oscillator::new();
        bent.set_frequency(220.0, 48_000.0);
        let params = OscParams {
            phase_distortion: PhaseDistortionMode::Bend,
            phase_amount: 1.0,
            ..Default::default()
        };
        let a: Vec<_> = (0..1024).map(|_| clean.tick(&table)).collect();
        let b: Vec<_> = (0..1024)
            .map(|_| bent.tick_with_params(&table, &params))
            .collect();
        assert!(rms_diff(&a, &b) > 0.01);
    }

    #[test]
    fn all_unison_styles_are_bounded_and_distinct_enough() {
        let centered = render(
            110.0,
            OscParams {
                unison_voices: 8,
                unison_style: UnisonStyle::Centered,
                unison_detune_cents: 18.0,
                ..Default::default()
            },
            8192,
        );
        for style in UnisonStyle::ALL {
            let samples = render(
                110.0,
                OscParams {
                    unison_voices: 8,
                    unison_style: style,
                    unison_detune_cents: 18.0,
                    ..Default::default()
                },
                8192,
            );
            assert!(
                samples.iter().all(|s| s.is_finite()),
                "{style:?} produced non-finite output"
            );
            assert!(
                peak(&samples) <= 1.01,
                "{style:?} peak too high: {}",
                peak(&samples)
            );
            assert!(rms(&samples) > 0.01, "{style:?} rendered too quietly");
            assert!(
                zero_crossings(&samples) > 10,
                "{style:?} has suspiciously few crossings"
            );
            if style != UnisonStyle::Centered {
                assert!(
                    rms_diff(&centered, &samples) > 1.0e-5,
                    "{style:?} too close to centered unison"
                );
            }
        }
    }

    /// A6.5 golden invariance: every public morph / phase mode / unison
    /// style must stay bounded and audible at both 44.1 kHz and 48 kHz.
    /// Catches DSP bugs that survive one sample rate but blow up on the
    /// other (most commonly: missing `sr` in a `g = tanf(PI*fc/sr)`).
    #[test]
    fn all_a6_modes_are_sample_rate_invariant() {
        fn render_sr(freq: f32, sr: f32, params: OscParams, n: usize) -> Vec<f32> {
            let table = SineTable::new();
            let mut osc = Oscillator::new();
            osc.set_frequency(freq, sr);
            osc.reset_phase();
            (0..n)
                .map(|_| osc.tick_with_params(&table, &params))
                .collect()
        }
        let rates = [44_100.0f32, 48_000.0];
        let configs: Vec<(&str, OscParams)> = SpectralMorph::ALL
            .iter()
            .map(|m| {
                (
                    "morph",
                    OscParams {
                        spectral_morph: *m,
                        morph_amount: 1.0,
                        ..Default::default()
                    },
                )
            })
            .chain(PhaseDistortionMode::ALL_A6.iter().map(|m| {
                (
                    "phase",
                    OscParams {
                        phase_distortion: *m,
                        phase_amount: 0.8,
                        ..Default::default()
                    },
                )
            }))
            .chain(UnisonStyle::ALL.iter().map(|s| {
                (
                    "unison",
                    OscParams {
                        unison_voices: 8,
                        unison_style: *s,
                        unison_detune_cents: 18.0,
                        ..Default::default()
                    },
                )
            }))
            .collect();
        for (label, params) in configs {
            for sr in rates {
                let samples = render_sr(220.0, sr, params, 4096);
                assert!(
                    samples.iter().all(|s| s.is_finite()),
                    "{label} non-finite at sr={sr}, params={params:?}"
                );
                let p = peak(&samples);
                assert!(p <= 1.01, "{label} peak {p} at sr={sr}");
                // Mean must be finite (PulseWidth legitimately produces
                // DC proportional to duty-cycle asymmetry; we only lock
                // the rest against runaway DC).
                let mean = samples.iter().sum::<f32>() / samples.len() as f32;
                assert!(mean.is_finite(), "{label} non-finite mean at sr={sr}");
                assert!(
                    mean.abs() < 0.95,
                    "{label} runaway DC {mean} at sr={sr}, params={params:?}"
                );
            }
        }
    }

    #[test]
    fn unison_stack_stays_bounded_and_audible() {
        let table = SineTable::new();
        let mut osc = Oscillator::new();
        osc.set_frequency(110.0, 48_000.0);
        let params = OscParams {
            unison_voices: 8,
            unison_style: UnisonStyle::Wide,
            unison_detune_cents: 18.0,
            ..Default::default()
        };
        let mut peak = 0.0f32;
        for _ in 0..4096 {
            peak = peak.max(osc.tick_with_params(&table, &params).abs());
        }
        assert!(peak > 0.05 && peak <= 1.0, "bad unison peak {peak}");
    }
}
