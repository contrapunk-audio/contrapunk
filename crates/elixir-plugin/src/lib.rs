//! Elixir VST3/CLAP plugin (Phase 21.B3 → B4).
//!
//! B3 introduced the minimal MIDI-triggered instrument shell. B4 expands
//! the parameter surface so DAWs can automate the current A6 core:
//! oscillator morph/phase/unison, amp ADSR, and filter model controls.

use std::num::NonZeroU32;
use std::sync::Arc;

use elixir_core::filter::FilterKind;
use elixir_core::osc::{PhaseDistortionMode, SpectralMorph, UnisonStyle};
use elixir_core::Engine;
use nih_plug::prelude::*;

// ── Plugin-facing enums ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginSpectralMorph {
    Passthrough,
    Vocode,
    #[name = "Form Scale"]
    FormScale,
    #[name = "Harmonic Scale"]
    HarmonicScale,
    #[name = "Inharmonic Scale"]
    InharmonicScale,
    Smear,
    #[name = "Random Amplitudes"]
    RandomAmplitudes,
    #[name = "Low Pass"]
    LowPass,
    #[name = "High Pass"]
    HighPass,
    #[name = "Phase Disperse"]
    PhaseDisperse,
    #[name = "Shepard Tone"]
    ShepardTone,
    Skew,
}

impl PluginSpectralMorph {
    fn to_core(self) -> SpectralMorph {
        match self {
            Self::Passthrough => SpectralMorph::Passthrough,
            Self::Vocode => SpectralMorph::Vocode,
            Self::FormScale => SpectralMorph::FormScale,
            Self::HarmonicScale => SpectralMorph::HarmonicScale,
            Self::InharmonicScale => SpectralMorph::InharmonicScale,
            Self::Smear => SpectralMorph::Smear,
            Self::RandomAmplitudes => SpectralMorph::RandomAmplitudes,
            Self::LowPass => SpectralMorph::LowPass,
            Self::HighPass => SpectralMorph::HighPass,
            Self::PhaseDisperse => SpectralMorph::PhaseDisperse,
            Self::ShepardTone => SpectralMorph::ShepardTone,
            Self::Skew => SpectralMorph::Skew,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginPhaseDistortion {
    Off,
    Quantize,
    Bend,
    Squeeze,
    Sync,
    #[name = "Pulse Width"]
    PulseWidth,
    #[name = "FM Osc A"]
    FmOscillatorA,
    #[name = "FM Osc B"]
    FmOscillatorB,
    #[name = "FM Sample"]
    FmSample,
    #[name = "RM Osc A"]
    RmOscillatorA,
    #[name = "RM Osc B"]
    RmOscillatorB,
    #[name = "RM Sample"]
    RmSample,
}

impl PluginPhaseDistortion {
    fn to_core(self) -> PhaseDistortionMode {
        match self {
            Self::Off => PhaseDistortionMode::Off,
            Self::Quantize => PhaseDistortionMode::Quantize,
            Self::Bend => PhaseDistortionMode::Bend,
            Self::Squeeze => PhaseDistortionMode::Squeeze,
            Self::Sync => PhaseDistortionMode::Sync,
            Self::PulseWidth => PhaseDistortionMode::PulseWidth,
            Self::FmOscillatorA => PhaseDistortionMode::FmOscillatorA,
            Self::FmOscillatorB => PhaseDistortionMode::FmOscillatorB,
            Self::FmSample => PhaseDistortionMode::FmSample,
            Self::RmOscillatorA => PhaseDistortionMode::RmOscillatorA,
            Self::RmOscillatorB => PhaseDistortionMode::RmOscillatorB,
            Self::RmSample => PhaseDistortionMode::RmSample,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginUnisonStyle {
    Centered,
    Octaves,
    Fifths,
    #[name = "Power Chord"]
    PowerChord,
    #[name = "Harmonic Series"]
    HarmonicSeries,
    Wide,
    Narrow,
    Organ,
    Suspended,
    Cluster,
    Alternating,
}

impl PluginUnisonStyle {
    fn to_core(self) -> UnisonStyle {
        match self {
            Self::Centered => UnisonStyle::Centered,
            Self::Octaves => UnisonStyle::Octaves,
            Self::Fifths => UnisonStyle::Fifths,
            Self::PowerChord => UnisonStyle::PowerChord,
            Self::HarmonicSeries => UnisonStyle::HarmonicSeries,
            Self::Wide => UnisonStyle::Wide,
            Self::Narrow => UnisonStyle::Narrow,
            Self::Organ => UnisonStyle::Organ,
            Self::Suspended => UnisonStyle::Suspended,
            Self::Cluster => UnisonStyle::Cluster,
            Self::Alternating => UnisonStyle::Alternating,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginFilterKind {
    #[name = "Digital SVF"]
    DigitalSvf,
    Diode,
    Dirty,
    Formant,
    Phaser,
}

impl PluginFilterKind {
    fn to_core(self) -> FilterKind {
        match self {
            Self::DigitalSvf => FilterKind::DigitalSvf,
            Self::Diode => FilterKind::Diode,
            Self::Dirty => FilterKind::Dirty,
            Self::Formant => FilterKind::Formant,
            Self::Phaser => FilterKind::Phaser,
        }
    }
}

// ── Parameters ───────────────────────────────────────────────────────

#[derive(Params)]
struct ElixirParams {
    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "amp_attack"]
    pub amp_attack: FloatParam,
    #[id = "amp_decay"]
    pub amp_decay: FloatParam,
    #[id = "amp_sustain"]
    pub amp_sustain: FloatParam,
    #[id = "amp_release"]
    pub amp_release: FloatParam,

    #[id = "spectral_morph"]
    pub spectral_morph: EnumParam<PluginSpectralMorph>,
    #[id = "morph_amount"]
    pub morph_amount: FloatParam,
    #[id = "phase_distortion"]
    pub phase_distortion: EnumParam<PluginPhaseDistortion>,
    #[id = "phase_amount"]
    pub phase_amount: FloatParam,
    #[id = "unison_style"]
    pub unison_style: EnumParam<PluginUnisonStyle>,
    #[id = "unison_voices"]
    pub unison_voices: IntParam,
    #[id = "unison_detune"]
    pub unison_detune: FloatParam,

    #[id = "filter_kind"]
    pub filter_kind: EnumParam<PluginFilterKind>,
    #[id = "filter_cutoff"]
    pub filter_cutoff: FloatParam,
    #[id = "filter_resonance"]
    pub filter_resonance: FloatParam,
    #[id = "filter_drive"]
    pub filter_drive: FloatParam,
    #[id = "filter_gain"]
    pub filter_gain: FloatParam,
    #[id = "filter_morph_x"]
    pub filter_morph_x: FloatParam,
    #[id = "filter_morph_y"]
    pub filter_morph_y: FloatParam,
}

impl Default for ElixirParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(-12.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-60.0),
                    max: util::db_to_gain(0.0),
                    factor: FloatRange::gain_skew_factor(-60.0, 0.0),
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            amp_attack: seconds_param("Attack", 0.005, 0.001, 4.0),
            amp_decay: seconds_param("Decay", 0.120, 0.001, 4.0),
            amp_sustain: percent_param("Sustain", 0.70),
            amp_release: seconds_param("Release", 0.250, 0.001, 8.0),

            spectral_morph: EnumParam::new("Spectral Morph", PluginSpectralMorph::Passthrough),
            morph_amount: percent_param("Morph Amount", 0.0),
            phase_distortion: EnumParam::new("Phase Distortion", PluginPhaseDistortion::Off),
            phase_amount: percent_param("Phase Amount", 0.0),
            unison_style: EnumParam::new("Unison Style", PluginUnisonStyle::Centered),
            unison_voices: IntParam::new("Unison Voices", 1, IntRange::Linear { min: 1, max: 16 }),
            unison_detune: FloatParam::new(
                "Unison Detune",
                8.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit(" cents"),

            filter_kind: EnumParam::new("Filter", PluginFilterKind::DigitalSvf),
            filter_cutoff: FloatParam::new(
                "Cutoff",
                8_000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: 0.35,
                },
            )
            .with_unit(" Hz"),
            filter_resonance: percent_param("Resonance", 0.0),
            filter_drive: FloatParam::new(
                "Filter Drive",
                1.0,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 32.0,
                    factor: 0.5,
                },
            ),
            filter_gain: FloatParam::new(
                "Filter Gain",
                1.0,
                FloatRange::Linear { min: 0.0, max: 4.0 },
            ),
            filter_morph_x: percent_param("Filter Morph X", 0.0),
            filter_morph_y: percent_param("Filter Morph Y", 0.0),
        }
    }
}

fn percent_param(name: &'static str, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 1.0 })
        .with_unit(" %")
        .with_value_to_string(formatters::v2s_f32_percentage(0))
        .with_string_to_value(formatters::s2v_f32_percentage())
}

fn seconds_param(name: &'static str, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min,
            max,
            factor: 0.35,
        },
    )
    .with_unit(" s")
}

struct ElixirPlugin {
    params: Arc<ElixirParams>,
    engine: Engine,
    sample_rate: f32,
    max_block: usize,
}

impl Default for ElixirPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ElixirParams::default()),
            engine: Engine::new(),
            sample_rate: 48_000.0,
            max_block: 2048,
        }
    }
}

impl ElixirPlugin {
    fn sync_params(&mut self) {
        self.engine.set_master_gain(self.params.gain.value());
        self.engine
            .set_amp_attack_secs(self.params.amp_attack.value());
        self.engine
            .set_amp_decay_secs(self.params.amp_decay.value());
        self.engine.set_amp_sustain(self.params.amp_sustain.value());
        self.engine
            .set_amp_release_secs(self.params.amp_release.value());

        self.engine
            .set_spectral_morph(self.params.spectral_morph.value().to_core());
        self.engine
            .set_morph_amount(self.params.morph_amount.value());
        self.engine
            .set_phase_distortion(self.params.phase_distortion.value().to_core());
        self.engine
            .set_phase_amount(self.params.phase_amount.value());
        self.engine
            .set_unison_style(self.params.unison_style.value().to_core());
        self.engine
            .set_unison_voices(self.params.unison_voices.value() as u8);
        self.engine
            .set_unison_detune_cents(self.params.unison_detune.value());

        self.engine
            .set_filter_kind(self.params.filter_kind.value().to_core());
        self.engine
            .set_filter_cutoff_hz(self.params.filter_cutoff.value());
        self.engine
            .set_filter_resonance(self.params.filter_resonance.value());
        self.engine
            .set_filter_drive(self.params.filter_drive.value());
        self.engine.set_filter_gain(self.params.filter_gain.value());
        self.engine.set_filter_morph(
            self.params.filter_morph_x.value(),
            self.params.filter_morph_y.value(),
        );
    }
}

impl Plugin for ElixirPlugin {
    const NAME: &'static str = "Elixir";
    const VENDOR: &'static str = "Contrapunk Audio";
    const URL: &'static str = "https://contrapunk.com";
    const EMAIL: &'static str = "hello@contrapunk.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.max_block = buffer_config.max_buffer_size as usize;
        self.engine
            .prepare(buffer_config.sample_rate as u32, self.max_block);
        true
    }

    fn reset(&mut self) {
        self.engine.all_notes_off();
        self.engine
            .prepare(self.sample_rate as u32, self.max_block.max(1));
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.sync_params();

        let mut next_event = context.next_event();
        for (sample_id, mut channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        let velocity = (velocity * 127.0).round().clamp(1.0, 127.0) as u8;
                        self.engine.note_on(note, velocity);
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        self.engine.note_off(note);
                    }
                    _ => {}
                }
                next_event = context.next_event();
            }

            let channels = channel_samples.len().min(2);
            if channels == 0 {
                continue;
            }
            let mut frame = [0.0f32; 2];
            self.engine.process(&mut frame[..channels], channels);
            for (sample, rendered) in channel_samples.iter_mut().zip(frame.iter()) {
                *sample = *rendered;
            }
        }

        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for ElixirPlugin {
    const CLAP_ID: &'static str = "com.contrapunk.elixir.plugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Elixir morphing synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for ElixirPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"ElixirSynth_v001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(ElixirPlugin);
nih_export_vst3!(ElixirPlugin);
