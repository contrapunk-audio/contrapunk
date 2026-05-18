//! Elixir VST3/CLAP plugin skeleton (Phase 21.B3).
//!
//! B3 keeps the surface deliberately small: a MIDI-triggered instrument
//! powered by `elixir_core::Engine`, exported as CLAP and VST3 through
//! nih-plug. B4 expands this into the full automatable parameter surface.

use std::num::NonZeroU32;
use std::sync::Arc;

use elixir_core::Engine;
use nih_plug::prelude::*;

#[derive(Params)]
struct ElixirParams {
    #[id = "gain"]
    pub gain: FloatParam,
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
        }
    }
}

struct ElixirPlugin {
    params: Arc<ElixirParams>,
    engine: Engine,
    sample_rate: f32,
}

impl Default for ElixirPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ElixirParams::default()),
            engine: Engine::new(),
            sample_rate: 48_000.0,
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
        self.engine.prepare(
            buffer_config.sample_rate as u32,
            buffer_config.max_buffer_size as usize,
        );
        true
    }

    fn reset(&mut self) {
        self.engine.all_notes_off();
        self.engine.prepare(self.sample_rate as u32, 2048);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set_master_gain(self.params.gain.value());

        let mut next_event = context.next_event();
        for (sample_id, mut channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        self.engine.note_on(note, (velocity * 127.0) as u8);
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        self.engine.note_off(note);
                    }
                    _ => {}
                }
                next_event = context.next_event();
            }

            let channels = channel_samples.len().min(2);
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
