//! Filters (Phase 21.A4 → A6).
//!
//! A4 shipped a Zavalishin TPT/ZDF state-variable lowpass. A6 keeps
//! that default intact and adds a fixed-state [`FilterModel`] enum for
//! the additional design-doc filter colors: diode, dirty, formant, and
//! phaser-filter. The model enum is static dispatch, stores all state
//! inline per voice, and allocates nothing in the audio path.

use core::f32::consts::PI;

use contrapunk_dsp::sat::quick_tanh;

/// User-facing filter model selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FilterKind {
    #[default]
    DigitalSvf,
    Diode,
    Dirty,
    Formant,
    Phaser,
}

impl FilterKind {
    pub const ALL: [Self; 5] = [
        Self::DigitalSvf,
        Self::Diode,
        Self::Dirty,
        Self::Formant,
        Self::Phaser,
    ];
}

/// Block-level filter parameters shared by every voice for one process
/// block. `morph_x/y` are used by the formant model as a vowel grid.
#[derive(Clone, Copy, Debug)]
pub struct FilterParams {
    pub kind: FilterKind,
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub drive: f32,
    pub gain: f32,
    pub morph_x: f32,
    pub morph_y: f32,
    pub sample_rate: f32,
}

impl FilterParams {
    pub fn digital_svf(cutoff_hz: f32, resonance: f32, sample_rate: f32) -> Self {
        Self {
            kind: FilterKind::DigitalSvf,
            cutoff_hz,
            resonance,
            drive: 1.0,
            gain: 1.0,
            morph_x: 0.0,
            morph_y: 0.0,
            sample_rate,
        }
    }

    pub fn svf_coeffs(&self) -> SvfCoeffs {
        SvfCoeffs::from_params(self.cutoff_hz, self.resonance, self.sample_rate)
    }

    /// Pre-compute every coefficient any [`FilterModel`] needs for the
    /// upcoming block. Call this once per process block so the per-sample
    /// hot path can skip `tanf` and the rest of the trig math.
    pub fn prepare_coeffs(&self) -> FilterCoeffs {
        FilterCoeffs::from_params(self)
    }
}

/// Per-block coefficients pre-computed once for the selected filter
/// kind. Construct via [`FilterParams::prepare_coeffs`] and feed into
/// [`FilterModel::tick_prepared`] for the rest of the block.
#[derive(Clone, Copy, Debug)]
pub enum FilterCoeffs {
    DigitalSvf(SvfCoeffs),
    Diode(DiodeCoeffs),
    Dirty(DirtyCoeffs),
    Formant(FormantCoeffs),
    Phaser(PhaserCoeffs),
}

impl FilterCoeffs {
    pub fn from_params(p: &FilterParams) -> Self {
        match p.kind {
            FilterKind::DigitalSvf => Self::DigitalSvf(SvfCoeffs::from_params(
                p.cutoff_hz,
                p.resonance,
                p.sample_rate,
            )),
            FilterKind::Diode => Self::Diode(DiodeCoeffs::from_params(p)),
            FilterKind::Dirty => Self::Dirty(DirtyCoeffs::from_params(p)),
            FilterKind::Formant => Self::Formant(FormantCoeffs::from_params(p)),
            FilterKind::Phaser => Self::Phaser(PhaserCoeffs::from_params(p)),
        }
    }

    pub fn kind(&self) -> FilterKind {
        match self {
            Self::DigitalSvf(_) => FilterKind::DigitalSvf,
            Self::Diode(_) => FilterKind::Diode,
            Self::Dirty(_) => FilterKind::Dirty,
            Self::Formant(_) => FilterKind::Formant,
            Self::Phaser(_) => FilterKind::Phaser,
        }
    }
}

#[inline]
fn g_from_cutoff(cutoff_hz: f32, sample_rate: f32) -> f32 {
    let sr = sample_rate.max(1.0);
    let fc = cutoff_hz.clamp(20.0, sr * 0.49);
    libm::tanf(PI * fc / sr)
}

#[derive(Clone, Copy, Debug)]
pub struct DiodeCoeffs {
    a: f32,
    drive: f32,
    feedback_gain: f32,
    gain: f32,
}

impl DiodeCoeffs {
    pub fn from_params(p: &FilterParams) -> Self {
        let g = g_from_cutoff(p.cutoff_hz, p.sample_rate);
        Self {
            a: g / (1.0 + g),
            drive: p.drive.clamp(0.1, 32.0),
            feedback_gain: p.resonance.clamp(0.0, 0.99) * 3.0,
            gain: p.gain,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DirtyCoeffs {
    a: f32,
    drive: f32,
    tuned_res: f32,
    drive_mix: f32,
    gain: f32,
}

impl DirtyCoeffs {
    pub fn from_params(p: &FilterParams) -> Self {
        let g = g_from_cutoff(p.cutoff_hz, p.sample_rate);
        let res = p.resonance.clamp(0.0, 0.99);
        let drive = p.drive.clamp(0.1, 32.0);
        Self {
            a: g / (1.0 + g),
            drive,
            tuned_res: res * 3.5 / 1.0_f32.max(0.25 * g + 0.97),
            drive_mix: 1.0 + res * drive,
            gain: p.gain,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FormantCoeffs {
    bands: [(SvfCoeffs, f32); 4],
    gain: f32,
}

impl FormantCoeffs {
    pub fn from_params(p: &FilterParams) -> Self {
        let vowel = vowel_bands(p.morph_x, p.morph_y);
        let scale = (p.cutoff_hz / 1000.0).clamp(0.5, 2.0);
        let bands = core::array::from_fn(|i| {
            let band = vowel[i];
            (
                SvfCoeffs::from_params(band.hz * scale, band.resonance, p.sample_rate),
                band.gain,
            )
        });
        Self {
            bands,
            gain: p.gain,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PhaserCoeffs {
    g: f32,
    dry_mul: f32,
    wet_mul: f32,
    drive: f32,
    gain: f32,
}

impl PhaserCoeffs {
    pub fn from_params(p: &FilterParams) -> Self {
        let t = g_from_cutoff(p.cutoff_hz, p.sample_rate);
        let g = (t / (1.0 + t)).clamp(0.02, 0.92);
        let res = p.resonance.clamp(0.0, 1.0);
        Self {
            g,
            dry_mul: 1.0 - res * 0.5,
            wet_mul: 0.5 + res * 0.5,
            drive: p.drive.max(1.0),
            gain: p.gain,
        }
    }
}

/// Per-block coefficients derived from `(cutoff_hz, resonance, sample_rate)`.
#[derive(Clone, Copy, Debug)]
pub struct SvfCoeffs {
    pub a1: f32,
    pub a2: f32,
    pub a3: f32,
    pub k: f32,
}

impl SvfCoeffs {
    /// Identity coefficients — filter is bypass.
    pub const fn identity() -> Self {
        Self {
            a1: 1.0,
            a2: 0.0,
            a3: 0.0,
            k: 2.0,
        }
    }

    /// Compute coefficients for a target cutoff (Hz), resonance
    /// (`0..1`), and sample rate. Cutoff is clamped to `[20, fs * 0.49]`
    /// for stability.
    pub fn from_params(cutoff_hz: f32, resonance: f32, sample_rate: f32) -> Self {
        let sr = sample_rate.max(1.0);
        let nyq = sr * 0.5;
        let fc = cutoff_hz.clamp(20.0, nyq - 200.0);
        let g = libm::tanf(PI * fc / sr);
        let r = resonance.clamp(0.0, 0.99);
        // `k` maps 0→2 (no self-oscillation) down toward 0 at high R.
        let k = 2.0 - 2.0 * r;
        let denom = 1.0 + g * (g + k);
        let a1 = 1.0 / denom;
        let a2 = g * a1;
        let a3 = g * a2;
        Self { a1, a2, a3, k }
    }
}

/// Two-state SVF carrying nothing but `ic1eq` / `ic2eq`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Svf {
    ic1eq: f32,
    ic2eq: f32,
}

impl Svf {
    pub const fn new() -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    #[inline]
    fn tick_core(&mut self, x: f32, c: &SvfCoeffs) -> (f32, f32, f32) {
        let v3 = x - self.ic2eq;
        let v1 = c.a1 * self.ic1eq + c.a2 * v3;
        let v2 = self.ic2eq + c.a2 * self.ic1eq + c.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        let low = v2;
        let band = v1;
        let high = x - c.k * band - low;
        (low, band, high)
    }

    /// Lowpass output. Returns the LP tap and updates state.
    #[inline]
    pub fn tick_lp(&mut self, x: f32, c: &SvfCoeffs) -> f32 {
        self.tick_core(x, c).0
    }

    /// Bandpass output. Used by the A6 formant filter.
    #[inline]
    pub fn tick_bp(&mut self, x: f32, c: &SvfCoeffs) -> f32 {
        self.tick_core(x, c).1
    }
}

/// Per-voice filter state. Static enum dispatch keeps model selection
/// explicit without `dyn` on the hot path.
#[derive(Clone, Copy, Debug)]
pub enum FilterModel {
    DigitalSvf(Svf),
    Diode(DiodeFilter),
    Dirty(DirtyFilter),
    Formant(FormantFilter),
    Phaser(PhaserFilter),
}

impl FilterModel {
    pub const fn new() -> Self {
        Self::DigitalSvf(Svf::new())
    }

    pub fn kind(&self) -> FilterKind {
        match self {
            Self::DigitalSvf(_) => FilterKind::DigitalSvf,
            Self::Diode(_) => FilterKind::Diode,
            Self::Dirty(_) => FilterKind::Dirty,
            Self::Formant(_) => FilterKind::Formant,
            Self::Phaser(_) => FilterKind::Phaser,
        }
    }

    pub fn set_kind(&mut self, kind: FilterKind) {
        if self.kind() == kind {
            return;
        }
        *self = match kind {
            FilterKind::DigitalSvf => Self::DigitalSvf(Svf::new()),
            FilterKind::Diode => Self::Diode(DiodeFilter::new()),
            FilterKind::Dirty => Self::Dirty(DirtyFilter::new()),
            FilterKind::Formant => Self::Formant(FormantFilter::new()),
            FilterKind::Phaser => Self::Phaser(PhaserFilter::new()),
        };
    }

    pub fn reset(&mut self) {
        match self {
            Self::DigitalSvf(f) => f.reset(),
            Self::Diode(f) => f.reset(),
            Self::Dirty(f) => f.reset(),
            Self::Formant(f) => f.reset(),
            Self::Phaser(f) => f.reset(),
        }
    }

    /// Per-sample tick. Re-derives coefficients every call — convenient
    /// for tests but **not** real-time-safe for `tanf`-heavy models.
    /// Use [`FilterModel::tick_prepared`] in the audio callback.
    #[inline]
    pub fn tick(&mut self, x: f32, params: &FilterParams) -> f32 {
        let coeffs = FilterCoeffs::from_params(params);
        self.tick_prepared(x, &coeffs)
    }

    /// Per-sample tick using pre-computed per-block coefficients. The
    /// hot path: zero `tanf`, zero allocation, zero per-sample
    /// `FilterParams` round-trip. If the prepared coeffs don't match
    /// the current model state the sample passes through untouched
    /// (graceful degrade rather than panic — the engine corrects on the
    /// next block).
    #[inline]
    pub fn tick_prepared(&mut self, x: f32, coeffs: &FilterCoeffs) -> f32 {
        match (self, coeffs) {
            (Self::DigitalSvf(f), FilterCoeffs::DigitalSvf(c)) => f.tick_lp(x, c),
            (Self::Diode(f), FilterCoeffs::Diode(c)) => f.tick_prepared(x, c),
            (Self::Dirty(f), FilterCoeffs::Dirty(c)) => f.tick_prepared(x, c),
            (Self::Formant(f), FilterCoeffs::Formant(c)) => f.tick_prepared(x, c),
            (Self::Phaser(f), FilterCoeffs::Phaser(c)) => f.tick_prepared(x, c),
            // Kind/coeffs out of sync. Engine resyncs on the next block.
            _ => x,
        }
    }
}

impl Default for FilterModel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DiodeFilter {
    stages: [f32; 4],
    hp_state: f32,
    hp_last: f32,
}

impl DiodeFilter {
    pub const fn new() -> Self {
        Self {
            stages: [0.0; 4],
            hp_state: 0.0,
            hp_last: 0.0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    #[inline]
    pub fn tick(&mut self, x: f32, params: &FilterParams) -> f32 {
        let c = DiodeCoeffs::from_params(params);
        self.tick_prepared(x, &c)
    }

    #[inline]
    pub fn tick_prepared(&mut self, x: f32, c: &DiodeCoeffs) -> f32 {
        let hp = x - self.hp_last + 0.995 * self.hp_state;
        self.hp_last = x;
        self.hp_state = hp;

        let feedback = self.stages[3] * c.feedback_gain;
        let mut y = hp - feedback;

        for i in 0..4 {
            let target = if i == 0 {
                libm::tanhf(y * c.drive)
            } else {
                0.5 * (self.stages[i - 1] + self.stages[i])
            };
            self.stages[i] += c.a * (target - self.stages[i]);
            y = self.stages[i];
        }

        y.clamp(-1.0, 1.0) * c.gain
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DirtyFilter {
    stages: [f32; 4],
}

impl DirtyFilter {
    pub const fn new() -> Self {
        Self { stages: [0.0; 4] }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    #[inline]
    pub fn tick(&mut self, x: f32, params: &FilterParams) -> f32 {
        let c = DirtyCoeffs::from_params(params);
        self.tick_prepared(x, &c)
    }

    #[inline]
    pub fn tick_prepared(&mut self, x: f32, c: &DirtyCoeffs) -> f32 {
        let mut y = (x - self.stages[3] * c.tuned_res) * c.drive_mix;

        for i in 0..2 {
            self.stages[i] += c.a * (y - self.stages[i]);
            y = self.stages[i];
        }
        for i in 2..4 {
            let sat = quick_tanh(y * c.drive);
            self.stages[i] += c.a * (sat - self.stages[i]);
            y = self.stages[i];
        }

        (y * c.gain).clamp(-2.0, 2.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FormantFilter {
    bands: [Svf; 4],
}

impl FormantFilter {
    pub const fn new() -> Self {
        Self {
            bands: [Svf::new(), Svf::new(), Svf::new(), Svf::new()],
        }
    }

    pub fn reset(&mut self) {
        for band in self.bands.iter_mut() {
            band.reset();
        }
    }

    #[inline]
    pub fn tick(&mut self, x: f32, params: &FilterParams) -> f32 {
        let c = FormantCoeffs::from_params(params);
        self.tick_prepared(x, &c)
    }

    #[inline]
    pub fn tick_prepared(&mut self, x: f32, c: &FormantCoeffs) -> f32 {
        let mut sum = 0.0;
        for (i, (coeffs, gain)) in c.bands.iter().enumerate() {
            sum += self.bands[i].tick_bp(x, coeffs) * *gain;
        }
        (sum * c.gain).clamp(-2.0, 2.0)
    }
}

#[derive(Clone, Copy, Debug)]
struct FormantBand {
    hz: f32,
    resonance: f32,
    gain: f32,
}

const VOWEL_A: [FormantBand; 4] = [
    FormantBand {
        hz: 730.0,
        resonance: 0.78,
        gain: 1.00,
    },
    FormantBand {
        hz: 1090.0,
        resonance: 0.72,
        gain: 0.55,
    },
    FormantBand {
        hz: 2440.0,
        resonance: 0.68,
        gain: 0.35,
    },
    FormantBand {
        hz: 3400.0,
        resonance: 0.62,
        gain: 0.20,
    },
];
const VOWEL_O: [FormantBand; 4] = [
    FormantBand {
        hz: 570.0,
        resonance: 0.82,
        gain: 1.00,
    },
    FormantBand {
        hz: 840.0,
        resonance: 0.76,
        gain: 0.52,
    },
    FormantBand {
        hz: 2410.0,
        resonance: 0.65,
        gain: 0.30,
    },
    FormantBand {
        hz: 3200.0,
        resonance: 0.60,
        gain: 0.18,
    },
];
const VOWEL_I: [FormantBand; 4] = [
    FormantBand {
        hz: 270.0,
        resonance: 0.88,
        gain: 0.95,
    },
    FormantBand {
        hz: 2290.0,
        resonance: 0.74,
        gain: 0.70,
    },
    FormantBand {
        hz: 3010.0,
        resonance: 0.66,
        gain: 0.32,
    },
    FormantBand {
        hz: 3600.0,
        resonance: 0.60,
        gain: 0.18,
    },
];
const VOWEL_E: [FormantBand; 4] = [
    FormantBand {
        hz: 530.0,
        resonance: 0.84,
        gain: 1.00,
    },
    FormantBand {
        hz: 1840.0,
        resonance: 0.76,
        gain: 0.65,
    },
    FormantBand {
        hz: 2480.0,
        resonance: 0.66,
        gain: 0.32,
    },
    FormantBand {
        hz: 3400.0,
        resonance: 0.60,
        gain: 0.18,
    },
];

fn vowel_bands(x: f32, y: f32) -> [FormantBand; 4] {
    let x = x.clamp(0.0, 1.0);
    let y = y.clamp(0.0, 1.0);
    core::array::from_fn(|i| {
        let top = lerp_band(VOWEL_A[i], VOWEL_O[i], x);
        let bottom = lerp_band(VOWEL_I[i], VOWEL_E[i], x);
        lerp_band(top, bottom, y)
    })
}

fn lerp_band(a: FormantBand, b: FormantBand, t: f32) -> FormantBand {
    FormantBand {
        hz: a.hz + (b.hz - a.hz) * t,
        resonance: a.resonance + (b.resonance - a.resonance) * t,
        gain: a.gain + (b.gain - a.gain) * t,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PhaserFilter {
    stages: [Allpass1; 12],
}

#[derive(Clone, Copy, Debug)]
struct Allpass1 {
    z: f32,
}

impl Allpass1 {
    const fn new() -> Self {
        Self { z: 0.0 }
    }

    #[inline]
    fn tick(&mut self, x: f32, g: f32) -> f32 {
        let y = -g * x + self.z;
        self.z = x + g * y;
        y
    }
}

impl PhaserFilter {
    pub const fn new() -> Self {
        Self {
            stages: [Allpass1::new(); 12],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    #[inline]
    pub fn tick(&mut self, x: f32, params: &FilterParams) -> f32 {
        let c = PhaserCoeffs::from_params(params);
        self.tick_prepared(x, &c)
    }

    #[inline]
    pub fn tick_prepared(&mut self, x: f32, c: &PhaserCoeffs) -> f32 {
        let mut y = x;
        let mut tap4 = 0.0;
        let mut tap8 = 0.0;
        let mut tap12 = 0.0;
        for i in 0..12 {
            y = self.stages[i].tick(y, c.g);
            if i == 3 {
                tap4 = y;
            } else if i == 7 {
                tap8 = y;
            } else if i == 11 {
                tap12 = y;
            }
        }
        let wet = tap4 * 0.35 + tap8 * 0.35 + tap12 * 0.30;
        let mixed = x * c.dry_mul + wet * c.wet_mul;
        quick_tanh(mixed * c.drive) * c.gain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(x: &[f32]) -> f32 {
        if x.is_empty() {
            return 0.0;
        }
        let s: f64 = x.iter().map(|v| (*v as f64).powi(2)).sum();
        (s / x.len() as f64).sqrt() as f32
    }

    fn render_model(kind: FilterKind, freq: f32, cutoff: f32) -> Vec<f32> {
        let sr = 48_000.0f32;
        let mut model = FilterModel::new();
        model.set_kind(kind);
        let params = FilterParams {
            kind,
            cutoff_hz: cutoff,
            resonance: 0.55,
            drive: 2.5,
            gain: 1.0,
            morph_x: 0.2,
            morph_y: 0.7,
            sample_rate: sr,
        };
        let omega = 2.0 * PI * freq / sr;
        let mut buf = vec![0.0f32; 4800];
        for (i, s) in buf.iter_mut().enumerate() {
            *s = model.tick(libm::sinf(omega * i as f32), &params);
        }
        buf
    }

    #[test]
    fn filter_model_default_matches_digital_svf_kind() {
        assert_eq!(FilterModel::new().kind(), FilterKind::DigitalSvf);
    }

    #[test]
    fn filter_model_switch_resets_kind() {
        let mut f = FilterModel::new();
        f.set_kind(FilterKind::Diode);
        assert_eq!(f.kind(), FilterKind::Diode);
        f.set_kind(FilterKind::Dirty);
        assert_eq!(f.kind(), FilterKind::Dirty);
        f.set_kind(FilterKind::DigitalSvf);
        assert_eq!(f.kind(), FilterKind::DigitalSvf);
    }

    #[test]
    fn lp_passes_low_frequencies() {
        let sr = 48_000.0f32;
        let c = SvfCoeffs::from_params(8_000.0, 0.0, sr);
        let mut svf = Svf::new();
        let mut buf = vec![0.0f32; 4800];
        let omega = 2.0 * PI * 100.0 / sr;
        for (i, s) in buf.iter_mut().enumerate() {
            *s = svf.tick_lp(libm::sinf(omega * i as f32), &c);
        }
        let settled = &buf[2400..];
        assert!(rms(settled) > 0.6, "100Hz LP rms too low: {}", rms(settled));
    }

    #[test]
    fn lp_attenuates_high_frequencies() {
        let sr = 48_000.0f32;
        let c = SvfCoeffs::from_params(500.0, 0.0, sr);
        let mut svf = Svf::new();
        let mut buf = vec![0.0f32; 4800];
        let omega = 2.0 * PI * 8_000.0 / sr;
        for (i, s) in buf.iter_mut().enumerate() {
            *s = svf.tick_lp(libm::sinf(omega * i as f32), &c);
        }
        let settled = &buf[2400..];
        assert!(
            rms(settled) < 0.15,
            "8kHz LP rms too high: {}",
            rms(settled)
        );
    }

    #[test]
    fn sweep_changes_amplitude_passed_through() {
        let sr = 48_000.0f32;
        let omega = 2.0 * PI * 2_000.0 / sr;

        let c_low = SvfCoeffs::from_params(500.0, 0.0, sr);
        let mut svf_low = Svf::new();
        let mut buf_low = vec![0.0f32; 4800];
        for (i, s) in buf_low.iter_mut().enumerate() {
            *s = svf_low.tick_lp(libm::sinf(omega * i as f32), &c_low);
        }

        let c_high = SvfCoeffs::from_params(8_000.0, 0.0, sr);
        let mut svf_high = Svf::new();
        let mut buf_high = vec![0.0f32; 4800];
        for (i, s) in buf_high.iter_mut().enumerate() {
            *s = svf_high.tick_lp(libm::sinf(omega * i as f32), &c_high);
        }

        let r_low = rms(&buf_low[2400..]);
        let r_high = rms(&buf_high[2400..]);
        assert!(
            r_high > r_low * 2.0,
            "expected open cutoff to pass much more signal: low={r_low}, high={r_high}"
        );
    }

    #[test]
    fn high_resonance_amplifies_signal_near_cutoff() {
        let sr = 48_000.0f32;
        let omega = 2.0 * PI * 1_000.0 / sr;

        let c_flat = SvfCoeffs::from_params(1_000.0, 0.0, sr);
        let mut svf_flat = Svf::new();
        let mut buf_flat = vec![0.0f32; 4800];
        for (i, s) in buf_flat.iter_mut().enumerate() {
            *s = svf_flat.tick_lp(libm::sinf(omega * i as f32), &c_flat);
        }

        let c_reso = SvfCoeffs::from_params(1_000.0, 0.9, sr);
        let mut svf_reso = Svf::new();
        let mut buf_reso = vec![0.0f32; 4800];
        for (i, s) in buf_reso.iter_mut().enumerate() {
            *s = svf_reso.tick_lp(libm::sinf(omega * i as f32), &c_reso);
        }

        let r_flat = rms(&buf_flat[2400..]);
        let r_reso = rms(&buf_reso[2400..]);
        assert!(
            r_reso > r_flat * 1.2,
            "expected resonance peak: flat={r_flat}, reso={r_reso}"
        );
    }

    #[test]
    fn diode_filter_attenuates_high_frequencies() {
        let low = render_model(FilterKind::Diode, 100.0, 800.0);
        let high = render_model(FilterKind::Diode, 8_000.0, 800.0);
        assert!(rms(&low[2400..]) > rms(&high[2400..]) * 2.0);
    }

    #[test]
    fn dirty_filter_remains_finite_at_high_drive() {
        let sr = 48_000.0;
        let mut model = FilterModel::Dirty(DirtyFilter::new());
        let params = FilterParams {
            kind: FilterKind::Dirty,
            cutoff_hz: 2_000.0,
            resonance: 0.95,
            drive: 32.0,
            gain: 1.0,
            morph_x: 0.0,
            morph_y: 0.0,
            sample_rate: sr,
        };
        for i in 0..4096 {
            let x = libm::sinf(2.0 * PI * 220.0 * i as f32 / sr) * 2.0;
            let y = model.tick(x, &params);
            assert!(y.is_finite());
            assert!(y.abs() <= 2.1);
        }
    }

    #[test]
    fn formant_filter_morph_changes_response() {
        let a = render_model(FilterKind::Formant, 730.0, 1_000.0);
        let sr = 48_000.0;
        let mut model = FilterModel::Formant(FormantFilter::new());
        let params = FilterParams {
            kind: FilterKind::Formant,
            cutoff_hz: 1_000.0,
            resonance: 0.5,
            drive: 1.0,
            gain: 1.0,
            morph_x: 1.0,
            morph_y: 1.0,
            sample_rate: sr,
        };
        let omega = 2.0 * PI * 730.0 / sr;
        let mut e = vec![0.0f32; 4800];
        for (i, s) in e.iter_mut().enumerate() {
            *s = model.tick(libm::sinf(omega * i as f32), &params);
        }
        let diff = a.iter().zip(&e).map(|(a, b)| (a - b).abs()).sum::<f32>() / a.len() as f32;
        assert!(
            diff > 1.0e-3,
            "formant morph did not change response: {diff}"
        );
    }

    #[test]
    fn prepared_path_matches_slow_path_for_all_kinds() {
        // A6.4: tick_prepared (per-block coeffs) must produce identical
        // output to tick (per-sample coeffs) given the same params.
        let sr = 48_000.0;
        let params_for = |kind: FilterKind| FilterParams {
            kind,
            cutoff_hz: 1_200.0,
            resonance: 0.65,
            drive: 3.5,
            gain: 1.0,
            morph_x: 0.3,
            morph_y: 0.7,
            sample_rate: sr,
        };
        for kind in FilterKind::ALL {
            let p = params_for(kind);
            let coeffs = p.prepare_coeffs();

            let mut slow = FilterModel::new();
            slow.set_kind(kind);
            let mut fast = FilterModel::new();
            fast.set_kind(kind);

            let omega = 2.0 * PI * 440.0 / sr;
            for i in 0..2048 {
                let x = libm::sinf(omega * i as f32);
                let a = slow.tick(x, &p);
                let b = fast.tick_prepared(x, &coeffs);
                assert!(
                    (a - b).abs() < 1.0e-5,
                    "prepared/slow drift for {kind:?} at i={i}: slow={a}, prepared={b}"
                );
            }
        }
    }

    #[test]
    fn prepared_coeffs_kind_matches_params_kind() {
        for kind in FilterKind::ALL {
            let p = FilterParams {
                kind,
                cutoff_hz: 1_000.0,
                resonance: 0.5,
                drive: 1.0,
                gain: 1.0,
                morph_x: 0.5,
                morph_y: 0.5,
                sample_rate: 48_000.0,
            };
            assert_eq!(p.prepare_coeffs().kind(), kind);
        }
    }

    #[test]
    fn phaser_filter_changes_response_with_cutoff() {
        let low = render_model(FilterKind::Phaser, 1_000.0, 300.0);
        let high = render_model(FilterKind::Phaser, 1_000.0, 6_000.0);
        let diff = low
            .iter()
            .zip(&high)
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>()
            / low.len() as f32;
        assert!(
            diff > 1.0e-3,
            "phaser cutoff did not alter response: {diff}"
        );
    }
}
