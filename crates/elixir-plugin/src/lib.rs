//! Independently buildable Elixir VST3/CLAP instrument.
//!
//! The sound path shares Elixir's Chapter 1 harmonic colour and Chapter 2
//! interaction, articulation, and expression model.

use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use elixir_core::{role_param, Engine, RolePatch, VoiceEvent, VoiceId, VoiceRole, MAX_POLYPHONY};
use elixir_preset::RolePatchState;
use nih_plug::params::persist::PersistentField;
use nih_plug::prelude::*;

mod editor;

struct AtomicRolePatch {
    parameters: [AtomicU32; role_param::COUNT as usize],
}

impl AtomicRolePatch {
    fn new(patch: RolePatch) -> Self {
        Self {
            parameters: std::array::from_fn(|index| {
                AtomicU32::new(patch.parameter(index as u8).unwrap_or(0.0).to_bits())
            }),
        }
    }

    fn load(&self) -> RolePatch {
        let mut patch = RolePatch::sine();
        for (index, parameter) in self.parameters.iter().enumerate() {
            patch.set_parameter(
                index as u8,
                f32::from_bits(parameter.load(Ordering::Relaxed)),
            );
        }
        patch
    }

    fn store(&self, patch: RolePatch) {
        let patch = patch.sanitized();
        for (index, parameter) in self.parameters.iter().enumerate() {
            parameter.store(
                patch.parameter(index as u8).unwrap_or(0.0).to_bits(),
                Ordering::Relaxed,
            );
        }
    }
}

impl<'a> PersistentField<'a, RolePatchState> for AtomicRolePatch {
    fn set(&self, value: RolePatchState) {
        self.store(value.to_core());
    }

    fn map<F, R>(&self, map: F) -> R
    where
        F: Fn(&RolePatchState) -> R,
    {
        let snapshot = RolePatchState::from(self.load());
        map(&snapshot)
    }
}

#[derive(Params)]
struct ElixirParams {
    #[id = "gain"]
    gain: FloatParam,

    #[persist = "role_patch_v1"]
    patch: AtomicRolePatch,

    #[persist = "webview_state"]
    webview_state: Arc<nih_plug_webview::WebViewState>,
}

impl Default for ElixirParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Master Gain",
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
            patch: AtomicRolePatch::new(RolePatch::sine()),
            webview_state: editor::default_webview_state(),
        }
    }
}

#[derive(Clone, Copy)]
struct HostOwner {
    core_id: VoiceId,
    host_id: Option<i32>,
    channel: u8,
    note: u8,
    age: u64,
}

struct ElixirPlugin {
    params: Arc<ElixirParams>,
    engine: Engine,
    owners: [Option<HostOwner>; MAX_POLYPHONY],
    next_id: u64,
    sample_rate: f32,
    max_block: usize,
    scratch: Vec<f32>,
    #[cfg(test)]
    process_calls: usize,
}

impl Default for ElixirPlugin {
    fn default() -> Self {
        let mut engine = Engine::new();
        engine.prepare(48_000, 2048);
        Self {
            params: Arc::new(ElixirParams::default()),
            engine,
            owners: [None; MAX_POLYPHONY],
            next_id: 0,
            sample_rate: 48_000.0,
            max_block: 2048,
            scratch: vec![0.0; 2048 * 2],
            #[cfg(test)]
            process_calls: 0,
        }
    }
}

impl ElixirPlugin {
    const HOST_VOICE_PREFIX: u64 = 1 << 61;

    fn note_on(&mut self, host_id: Option<i32>, channel: u8, note: u8, velocity: f32) {
        if note >= 128 || !velocity.is_finite() {
            return;
        }
        if velocity <= 0.0 {
            self.note_off(host_id, channel, note, false);
            return;
        }

        let age = self.next_id;
        let core_id = VoiceId::new(Self::HOST_VOICE_PREFIX | age);
        self.next_id = self.next_id.wrapping_add(1) & (Self::HOST_VOICE_PREFIX - 1);
        let index = self
            .owners
            .iter()
            .position(Option::is_none)
            .or_else(|| {
                self.owners
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, owner)| owner.unwrap().age)
                    .map(|(index, _)| index)
            })
            .unwrap_or(0);
        if let Some(stolen) = self.owners[index] {
            self.engine.handle_voice_event(VoiceEvent::NoteOff {
                voice_id: stolen.core_id,
            });
        }
        self.owners[index] = Some(HostOwner {
            core_id,
            host_id,
            channel,
            note,
            age,
        });
        self.engine.handle_voice_event(VoiceEvent::NoteOn {
            voice_id: core_id,
            role: VoiceRole::Input,
            midi_anchor: note,
            frequency_hz: elixir_core::util::midi_to_freq(note),
            velocity: (velocity * 127.0).round().clamp(1.0, 127.0) as u8,
        });
    }

    fn note_off(&mut self, host_id: Option<i32>, channel: u8, note: u8, all: bool) {
        loop {
            let match_owner = |owner: &HostOwner| match host_id {
                Some(id) => owner.host_id == Some(id),
                None => owner.channel == channel && owner.note == note,
            };
            let found = self
                .owners
                .iter()
                .enumerate()
                .filter_map(|(index, owner)| owner.map(|owner| (index, owner)))
                .filter(|(_, owner)| match_owner(owner))
                .min_by_key(|(_, owner)| owner.age);
            let Some((index, owner)) = found else {
                break;
            };
            self.owners[index] = None;
            self.engine.handle_voice_event(VoiceEvent::NoteOff {
                voice_id: owner.core_id,
            });
            if !all || host_id.is_some() {
                break;
            }
        }
    }

    fn panic(&mut self) {
        self.owners.fill(None);
        self.engine.handle_voice_event(VoiceEvent::Panic);
    }

    fn handle_note_event(&mut self, event: NoteEvent<()>) {
        match event {
            NoteEvent::NoteOn {
                voice_id,
                channel,
                note,
                velocity,
                ..
            } => self.note_on(voice_id, channel, note, velocity),
            NoteEvent::NoteOff {
                voice_id,
                channel,
                note,
                ..
            } => self.note_off(voice_id, channel, note, false),
            NoteEvent::Choke {
                voice_id,
                channel,
                note,
                ..
            } => self.note_off(voice_id, channel, note, true),
            NoteEvent::MidiCC { cc: 1, value, .. } => self.engine.set_mod_wheel(value),
            NoteEvent::MidiCC { cc: 11, value, .. } => self.engine.set_expression(value),
            NoteEvent::MidiCC { cc: 64, value, .. } => self.engine.set_sustain_pedal(value >= 0.5),
            NoteEvent::MidiCC { cc: 120 | 123, .. } => self.panic(),
            NoteEvent::MidiPitchBend { value, .. } => self
                .engine
                .set_pitch_bend_cents((value.clamp(0.0, 1.0) - 0.5) * 400.0),
            NoteEvent::MidiChannelPressure { pressure, .. } => self.engine.set_expression(pressure),
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
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(editor::create_editor(
            self.params.clone(),
            &self.params.webview_state,
        )))
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
        self.panic();
        self.engine
            .prepare(buffer_config.sample_rate as u32, self.max_block.max(1));
        true
    }

    fn reset(&mut self) {
        self.panic();
        self.engine
            .prepare(self.sample_rate as u32, self.max_block.max(1));
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set_master_gain(self.params.gain.value());
        self.engine
            .set_role_patch(VoiceRole::Input, self.params.patch.load());
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
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("16-voice harmonic foundations synthesizer");
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

    fn note_on(timing: u32, voice_id: Option<i32>) -> NoteEvent<()> {
        NoteEvent::NoteOn {
            timing,
            voice_id,
            channel: 0,
            note: 69,
            velocity: 1.0,
        }
    }

    fn note_off(timing: u32, voice_id: Option<i32>) -> NoteEvent<()> {
        NoteEvent::NoteOff {
            timing,
            voice_id,
            channel: 0,
            note: 69,
            velocity: 0.0,
        }
    }

    #[test]
    fn sample_accurate_boundaries_split_without_allocating() {
        let mut plugin = ElixirPlugin::default();
        let mut left = [0.0; 128];
        let mut right = [0.0; 128];
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        let mut events = [note_on(0, Some(1)), note_off(64, Some(1))].into_iter();
        let first = events.next();
        assert_no_alloc::assert_no_alloc(|| {
            plugin.process_buffer_with_events(&mut outputs, 128, first, || events.next());
        });
        assert_eq!(plugin.process_calls, 2);
        assert!(left.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn absent_host_ids_preserve_repeated_note_fifo_ownership() {
        let mut plugin = ElixirPlugin::default();
        plugin.handle_note_event(note_on(0, None));
        plugin.handle_note_event(note_on(0, None));
        assert_eq!(plugin.engine.live_voice_count(), 2);
        plugin.handle_note_event(note_off(0, None));
        assert_eq!(plugin.engine.live_voice_count(), 1);
        plugin.handle_note_event(note_off(0, None));
        assert_eq!(plugin.engine.live_voice_count(), 0);
    }

    #[test]
    fn sustain_panic_and_reset_clear_ownership() {
        let mut plugin = ElixirPlugin::default();
        plugin.handle_note_event(note_on(0, Some(7)));
        plugin.handle_note_event(NoteEvent::MidiCC {
            timing: 0,
            channel: 0,
            cc: 64,
            value: 1.0,
        });
        plugin.handle_note_event(note_off(0, Some(7)));
        assert!(plugin.engine.sustain_pedal());
        plugin.handle_note_event(NoteEvent::MidiCC {
            timing: 0,
            channel: 0,
            cc: 123,
            value: 0.0,
        });
        assert_eq!(plugin.engine.live_voice_count(), 0);
        assert!(plugin.owners.iter().all(Option::is_none));
        Plugin::reset(&mut plugin);
        assert_eq!(plugin.engine.live_voice_count(), 0);
        assert!(plugin.owners.iter().all(Option::is_none));
    }

    #[test]
    fn persisted_patch_and_expression_reach_the_engine() {
        let params = ElixirParams::default();
        let mut patch = RolePatch::sine();
        patch.harmonics.amplitudes = [1.0, 0.5, 0.25, 0.0, 0.0, 0.0];
        patch.secondary.mode = elixir_core::CombineMode::Add;
        let state = RolePatchState::from(patch);
        PersistentField::set(&params.patch, state);
        assert_eq!(params.patch.load(), patch);

        let mut plugin = ElixirPlugin::default();
        plugin.handle_note_event(note_on(0, Some(1)));
        plugin.handle_note_event(NoteEvent::MidiChannelPressure {
            timing: 0,
            channel: 0,
            pressure: 0.0,
        });
        let mut output = [1.0; 64];
        plugin.engine.process(&mut output, 1);
        assert!(output.iter().all(|sample| sample.abs() < 1.0e-6));
        plugin.handle_note_event(NoteEvent::MidiPitchBend {
            timing: 0,
            channel: 0,
            value: 0.75,
        });
        assert_eq!(plugin.engine.pitch_bend_cents(), 100.0);
    }

    #[test]
    fn oversized_blocks_use_preallocated_chunks() {
        let mut plugin = ElixirPlugin {
            scratch: vec![0.0; 16],
            ..Default::default()
        };
        let capacity = plugin.scratch.capacity();
        let mut left = [0.0; 100];
        let mut right = [0.0; 100];
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        assert_no_alloc::assert_no_alloc(|| {
            plugin
                .process_buffer_with_events(&mut outputs, 100, Some(note_on(0, Some(1))), || None);
        });
        assert_eq!(plugin.scratch.capacity(), capacity);
        assert!(left.iter().any(|sample| sample.abs() > 1.0e-6));
    }
}

nih_export_clap!(ElixirPlugin);
nih_export_vst3!(ElixirPlugin);
