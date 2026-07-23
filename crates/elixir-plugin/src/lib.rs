//! Elixir VST3/CLAP plugin (Phase 21.B3 → B4).
//!
//! B3 introduced the minimal MIDI-triggered instrument shell. B4 expands
//! the parameter surface so DAWs can automate the current A6 core:
//! oscillator morph/phase/unison, amp ADSR, filters, and the FX chain.

use std::num::NonZeroU32;
use std::sync::Arc;

use elixir_core::filter::FilterKind;
use elixir_core::fx::{
    Chorus, Compressor, Delay, Drive, FdnReverb, Flanger, FxSlot, Phaser, Reverb,
};
use elixir_core::osc::{PhaseDistortionMode, SpectralMorph, UnisonStyle};
use elixir_core::{Engine, VoiceEvent, VoiceId, VoiceRole};
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

    #[id = "drive_on"]
    pub drive_on: BoolParam,
    #[id = "drive_amount"]
    pub drive_amount: FloatParam,
    #[id = "drive_mix"]
    pub drive_mix: FloatParam,

    #[id = "delay_on"]
    pub delay_on: BoolParam,
    #[id = "delay_time"]
    pub delay_time: FloatParam,
    #[id = "delay_feedback"]
    pub delay_feedback: FloatParam,
    #[id = "delay_mix"]
    pub delay_mix: FloatParam,

    #[id = "reverb_on"]
    pub reverb_on: BoolParam,
    #[id = "reverb_decay"]
    pub reverb_decay: FloatParam,
    #[id = "reverb_damping"]
    pub reverb_damping: FloatParam,
    #[id = "reverb_mix"]
    pub reverb_mix: FloatParam,

    #[id = "fdn_on"]
    pub fdn_on: BoolParam,
    #[id = "fdn_decay"]
    pub fdn_decay: FloatParam,
    #[id = "fdn_damping"]
    pub fdn_damping: FloatParam,
    #[id = "fdn_mix"]
    pub fdn_mix: FloatParam,

    #[id = "chorus_on"]
    pub chorus_on: BoolParam,
    #[id = "chorus_rate"]
    pub chorus_rate: FloatParam,
    #[id = "chorus_depth"]
    pub chorus_depth: FloatParam,
    #[id = "chorus_mix"]
    pub chorus_mix: FloatParam,

    #[id = "flanger_on"]
    pub flanger_on: BoolParam,
    #[id = "flanger_rate"]
    pub flanger_rate: FloatParam,
    #[id = "flanger_depth"]
    pub flanger_depth: FloatParam,
    #[id = "flanger_feedback"]
    pub flanger_feedback: FloatParam,
    #[id = "flanger_mix"]
    pub flanger_mix: FloatParam,

    #[id = "phaser_fx_on"]
    pub phaser_fx_on: BoolParam,
    #[id = "phaser_fx_rate"]
    pub phaser_fx_rate: FloatParam,
    #[id = "phaser_fx_depth"]
    pub phaser_fx_depth: FloatParam,
    #[id = "phaser_fx_feedback"]
    pub phaser_fx_feedback: FloatParam,
    #[id = "phaser_fx_mix"]
    pub phaser_fx_mix: FloatParam,

    #[id = "comp_on"]
    pub comp_on: BoolParam,
    #[id = "comp_threshold"]
    pub comp_threshold: FloatParam,
    #[id = "comp_ratio"]
    pub comp_ratio: FloatParam,
    #[id = "comp_attack"]
    pub comp_attack: FloatParam,
    #[id = "comp_release"]
    pub comp_release: FloatParam,
    #[id = "comp_makeup"]
    pub comp_makeup: FloatParam,
    #[id = "comp_mix"]
    pub comp_mix: FloatParam,
}

impl Default for ElixirParams {
    fn default() -> Self {
        Self {
            gain: gain_param("Gain", -12.0, -60.0, 0.0),

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

            drive_on: BoolParam::new("Drive", false),
            drive_amount: FloatParam::new(
                "Drive Amount",
                2.5,
                FloatRange::Linear {
                    min: 0.5,
                    max: 20.0,
                },
            ),
            drive_mix: percent_param("Drive Mix", 0.4),

            delay_on: BoolParam::new("Delay", false),
            delay_time: seconds_param("Delay Time", 0.375, 0.001, 2.0),
            delay_feedback: percent_param("Delay Feedback", 0.45),
            delay_mix: percent_param("Delay Mix", 0.30),

            reverb_on: BoolParam::new("Reverb", false),
            reverb_decay: percent_param("Reverb Decay", 0.85),
            reverb_damping: percent_param("Reverb Damping", 0.40),
            reverb_mix: percent_param("Reverb Mix", 0.30),

            fdn_on: BoolParam::new("FDN Reverb", false),
            fdn_decay: seconds_param("FDN Decay", 2.8, 0.2, 20.0),
            fdn_damping: percent_param("FDN Damping", 0.35),
            fdn_mix: percent_param("FDN Mix", 0.35),

            chorus_on: BoolParam::new("Chorus", false),
            chorus_rate: hz_param("Chorus Rate", 0.35, 0.01, 8.0),
            chorus_depth: ms_param("Chorus Depth", 8.0, 0.0, 40.0),
            chorus_mix: percent_param("Chorus Mix", 0.35),

            flanger_on: BoolParam::new("Flanger", false),
            flanger_rate: hz_param("Flanger Rate", 0.18, 0.01, 8.0),
            flanger_depth: ms_param("Flanger Depth", 2.5, 0.0, 10.0),
            flanger_feedback: percent_param("Flanger Feedback", 0.45),
            flanger_mix: percent_param("Flanger Mix", 0.40),

            phaser_fx_on: BoolParam::new("Phaser FX", false),
            phaser_fx_rate: hz_param("Phaser Rate", 0.20, 0.01, 8.0),
            phaser_fx_depth: percent_param("Phaser Depth", 0.75),
            phaser_fx_feedback: percent_param("Phaser Feedback", 0.65),
            phaser_fx_mix: percent_param("Phaser Mix", 0.45),

            comp_on: BoolParam::new("Compressor", false),
            comp_threshold: db_linear_param("Comp Threshold", -18.0, -60.0, 0.0),
            comp_ratio: FloatParam::new(
                "Comp Ratio",
                4.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 40.0,
                },
            ),
            comp_attack: ms_param("Comp Attack", 8.0, 0.1, 500.0),
            comp_release: ms_param("Comp Release", 120.0, 1.0, 2000.0),
            comp_makeup: db_linear_param("Comp Makeup", 4.0, -24.0, 24.0),
            comp_mix: percent_param("Comp Mix", 1.0),
        }
    }
}

fn gain_param(name: &'static str, default_db: f32, min_db: f32, max_db: f32) -> FloatParam {
    FloatParam::new(
        name,
        util::db_to_gain(default_db),
        FloatRange::Skewed {
            min: util::db_to_gain(min_db),
            max: util::db_to_gain(max_db),
            factor: FloatRange::gain_skew_factor(min_db, max_db),
        },
    )
    .with_unit(" dB")
    .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
    .with_string_to_value(formatters::s2v_f32_gain_to_db())
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

fn ms_param(name: &'static str, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max }).with_unit(" ms")
}

fn hz_param(name: &'static str, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min,
            max,
            factor: 0.35,
        },
    )
    .with_unit(" Hz")
}

fn db_linear_param(name: &'static str, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max }).with_unit(" dB")
}

struct ElixirPlugin {
    params: Arc<ElixirParams>,
    engine: Engine,
    sample_rate: f32,
    max_block: usize,
    scratch: Vec<f32>,
    #[cfg(test)]
    process_calls: usize,
}

impl Default for ElixirPlugin {
    fn default() -> Self {
        let mut plugin = Self {
            params: Arc::new(ElixirParams::default()),
            engine: Engine::new(),
            sample_rate: 48_000.0,
            max_block: 2048,
            scratch: vec![0.0; 2048 * 2],
            #[cfg(test)]
            process_calls: 0,
        };
        plugin.engine.prepare(48_000, 2048);
        plugin.install_fx_chain();
        plugin.sync_params();
        plugin
    }
}

impl ElixirPlugin {
    const HOST_VOICE_PREFIX: u64 = 1 << 61;
    const HOST_ID_FLAG: u64 = 1 << 32;

    fn host_voice_id(voice_id: Option<i32>, channel: u8, note: u8) -> VoiceId {
        let value = match voice_id {
            Some(id) => Self::HOST_ID_FLAG | id as u32 as u64,
            None => ((channel as u64) << 7) | note as u64,
        };
        VoiceId::new(Self::HOST_VOICE_PREFIX | value)
    }

    fn handle_note_event(&mut self, event: NoteEvent<()>) {
        match event {
            NoteEvent::NoteOn {
                voice_id,
                channel,
                note,
                velocity,
                ..
            } => {
                let velocity = (velocity * 127.0).round().clamp(1.0, 127.0) as u8;
                self.engine.handle_voice_event(VoiceEvent::NoteOn {
                    voice_id: Self::host_voice_id(voice_id, channel, note),
                    role: VoiceRole::Input,
                    midi_anchor: note,
                    frequency_hz: elixir_core::util::midi_to_freq(note),
                    velocity,
                });
            }
            NoteEvent::NoteOff {
                voice_id,
                channel,
                note,
                ..
            }
            | NoteEvent::Choke {
                voice_id,
                channel,
                note,
                ..
            } => self.engine.handle_voice_event(VoiceEvent::NoteOff {
                voice_id: Self::host_voice_id(voice_id, channel, note),
            }),
            _ => {}
        }
    }

    fn render_range(
        &mut self,
        outputs: &mut [&mut [f32]],
        channels: usize,
        start: usize,
        end: usize,
    ) {
        let frames = end - start;
        #[cfg(test)]
        {
            self.process_calls += 1;
        }
        let scratch = &mut self.scratch[..frames * channels];
        scratch.fill(0.0);
        self.engine.process(scratch, channels);
        for frame in 0..frames {
            for channel in 0..channels {
                outputs[channel][start + frame] = scratch[frame * channels + channel];
            }
        }
    }

    fn process_buffer_with_events(
        &mut self,
        outputs: &mut [&mut [f32]],
        frames: usize,
        mut event: Option<NoteEvent<()>>,
        mut next_event: impl FnMut() -> Option<NoteEvent<()>>,
    ) {
        let channels = outputs.len().min(2);
        if channels == 0 {
            while let Some(current) = event {
                if current.timing() as usize >= frames {
                    break;
                }
                self.handle_note_event(current);
                event = next_event();
            }
            return;
        }

        let max_frames = self.scratch.len() / channels;
        if max_frames == 0 {
            for channel in outputs.iter_mut().take(channels) {
                channel[..frames].fill(0.0);
            }
            return;
        }

        let mut cursor = 0;
        while cursor < frames {
            while event.is_some_and(|current| current.timing() as usize <= cursor) {
                self.handle_note_event(event.take().unwrap());
                event = next_event();
            }
            let event_frame = event
                .map(|current| current.timing() as usize)
                .unwrap_or(frames)
                .min(frames);
            let end = event_frame.min(cursor.saturating_add(max_frames));
            if end > cursor {
                self.render_range(outputs, channels, cursor, end);
                cursor = end;
            }
        }
    }

    fn install_fx_chain(&mut self) {
        let sr = self.sample_rate.max(1.0);
        let mut delay = Delay::new((sr * 2.0) as usize);
        delay.set_mix(0.0);
        let mut reverb = Reverb::new(sr);
        reverb.set_mix(0.0);
        let mut fdn = FdnReverb::new(sr);
        fdn.set_mix(0.0);
        let mut chorus = Chorus::new(sr);
        chorus.set_mix(0.0);
        let mut flanger = Flanger::new(sr);
        flanger.set_mix(0.0);
        let mut phaser = Phaser::new(sr);
        phaser.set_mix(0.0);
        let mut compressor = Compressor::new(sr);
        compressor.set_mix(0.0);

        self.engine.set_fx_slot(0, FxSlot::Drive(Drive::new()));
        self.engine.set_fx_slot(1, FxSlot::Delay(delay));
        self.engine.set_fx_slot(2, FxSlot::Reverb(reverb));
        self.engine.set_fx_slot(3, FxSlot::FdnReverb(fdn));
        self.engine.set_fx_slot(4, FxSlot::Chorus(chorus));
        self.engine.set_fx_slot(5, FxSlot::Flanger(flanger));
        self.engine.set_fx_slot(6, FxSlot::Phaser(phaser));
        self.engine.set_fx_slot(7, FxSlot::Compressor(compressor));
    }

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

        self.sync_fx_params();
    }

    fn sync_fx_params(&mut self) {
        if let FxSlot::Drive(drive) = &mut self.engine.fx_chain[0] {
            drive.set_drive(self.params.drive_amount.value());
            drive.set_mix(if self.params.drive_on.value() {
                self.params.drive_mix.value()
            } else {
                0.0
            });
        }
        if let FxSlot::Delay(delay) = &mut self.engine.fx_chain[1] {
            delay.set_delay_secs(self.params.delay_time.value(), self.sample_rate);
            delay.set_feedback(self.params.delay_feedback.value());
            delay.set_mix(if self.params.delay_on.value() {
                self.params.delay_mix.value()
            } else {
                0.0
            });
        }
        if let FxSlot::Reverb(reverb) = &mut self.engine.fx_chain[2] {
            reverb.set_decay(self.params.reverb_decay.value());
            reverb.set_damping(self.params.reverb_damping.value());
            reverb.set_mix(if self.params.reverb_on.value() {
                self.params.reverb_mix.value()
            } else {
                0.0
            });
        }
        if let FxSlot::FdnReverb(fdn) = &mut self.engine.fx_chain[3] {
            fdn.set_decay_seconds(self.params.fdn_decay.value());
            fdn.set_damping(self.params.fdn_damping.value());
            fdn.set_mix(if self.params.fdn_on.value() {
                self.params.fdn_mix.value()
            } else {
                0.0
            });
        }
        if let FxSlot::Chorus(chorus) = &mut self.engine.fx_chain[4] {
            chorus.set_rate_hz(self.params.chorus_rate.value());
            chorus.set_depth_ms(self.params.chorus_depth.value());
            chorus.set_mix(if self.params.chorus_on.value() {
                self.params.chorus_mix.value()
            } else {
                0.0
            });
        }
        if let FxSlot::Flanger(flanger) = &mut self.engine.fx_chain[5] {
            flanger.set_rate_hz(self.params.flanger_rate.value());
            flanger.set_depth_ms(self.params.flanger_depth.value());
            flanger.set_feedback(self.params.flanger_feedback.value());
            flanger.set_mix(if self.params.flanger_on.value() {
                self.params.flanger_mix.value()
            } else {
                0.0
            });
        }
        if let FxSlot::Phaser(phaser) = &mut self.engine.fx_chain[6] {
            phaser.set_rate_hz(self.params.phaser_fx_rate.value());
            phaser.set_depth(self.params.phaser_fx_depth.value());
            phaser.set_feedback(self.params.phaser_fx_feedback.value());
            phaser.set_mix(if self.params.phaser_fx_on.value() {
                self.params.phaser_fx_mix.value()
            } else {
                0.0
            });
        }
        if let FxSlot::Compressor(comp) = &mut self.engine.fx_chain[7] {
            comp.set_threshold_db(self.params.comp_threshold.value());
            comp.set_ratio(self.params.comp_ratio.value());
            comp.set_attack_ms(self.params.comp_attack.value());
            comp.set_release_ms(self.params.comp_release.value());
            comp.set_makeup_db(self.params.comp_makeup.value());
            comp.set_mix(if self.params.comp_on.value() {
                self.params.comp_mix.value()
            } else {
                0.0
            });
        }
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
        self.scratch.resize(self.max_block.max(1) * 2, 0.0);
        self.engine
            .prepare(buffer_config.sample_rate as u32, self.max_block);
        self.install_fx_chain();
        self.sync_params();
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

        let frames = buffer.samples();
        let first_event = context.next_event();
        self.process_buffer_with_events(buffer.as_slice(), frames, first_event, || {
            context.next_event()
        });

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

#[cfg(test)]
mod tests {
    use super::*;

    fn note_on(timing: u32, voice_id: i32) -> NoteEvent<()> {
        NoteEvent::NoteOn {
            timing,
            voice_id: Some(voice_id),
            channel: 0,
            note: 69,
            velocity: 1.0,
        }
    }

    fn note_off(timing: u32, voice_id: i32) -> NoteEvent<()> {
        NoteEvent::NoteOff {
            timing,
            voice_id: Some(voice_id),
            channel: 0,
            note: 69,
            velocity: 0.0,
        }
    }

    #[test]
    fn segmented_blocks_match_sample_processing_at_event_boundaries() {
        let mut block = ElixirPlugin::default();
        let mut left = [0.0; 128];
        let mut right = [0.0; 128];
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        let mut events = [note_on(0, 1), note_off(64, 1)].into_iter();
        let first = events.next();
        block.process_buffer_with_events(&mut outputs, 128, first, || events.next());
        assert_eq!(block.process_calls, 2);

        let mut sample = ElixirPlugin::default();
        let mut expected_left = [0.0; 128];
        let mut expected_right = [0.0; 128];
        for frame in 0..128 {
            if frame == 0 {
                sample.handle_note_event(note_on(0, 1));
            } else if frame == 64 {
                sample.handle_note_event(note_off(64, 1));
            }
            let mut rendered = [0.0; 2];
            sample.engine.process(&mut rendered, 2);
            expected_left[frame] = rendered[0];
            expected_right[frame] = rendered[1];
        }

        for (actual, expected) in left.iter().zip(expected_left) {
            assert!((actual - expected).abs() < 1.0e-6);
        }
        for (actual, expected) in right.iter().zip(expected_right) {
            assert!((actual - expected).abs() < 1.0e-6);
        }
    }

    #[test]
    fn oversized_host_block_is_processed_in_preallocated_chunks() {
        let mut plugin = ElixirPlugin::default();
        plugin.scratch = vec![0.0; 16];
        plugin.process_calls = 0;
        let scratch_capacity = plugin.scratch.capacity();
        let mut left = [0.0; 100];
        let mut right = [0.0; 100];
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        assert_no_alloc::assert_no_alloc(|| {
            plugin.process_buffer_with_events(&mut outputs, 100, Some(note_on(0, 1)), || None);
        });

        assert_eq!(plugin.process_calls, 13);
        assert_eq!(plugin.scratch.capacity(), scratch_capacity);
        assert!(left.iter().all(|sample| sample.is_finite()));
        assert!(left.iter().any(|sample| sample.abs() > 1.0e-6));
    }

    #[test]
    fn host_voice_ids_release_overlapping_same_note_independently() {
        let mut plugin = ElixirPlugin::default();
        plugin.handle_note_event(note_on(0, 10));
        plugin.handle_note_event(note_on(0, 11));
        assert_eq!(plugin.engine.live_voice_count(), 2);

        plugin.handle_note_event(note_off(0, 10));
        let mut left = [0.0; 64];
        let mut right = [0.0; 64];
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        plugin.process_buffer_with_events(&mut outputs, 64, None, || None);
        assert_eq!(plugin.engine.active_voice_count(), 1);
    }
}

nih_export_clap!(ElixirPlugin);
nih_export_vst3!(ElixirPlugin);
