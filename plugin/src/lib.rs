//! Contrapunk VST3/CLAP plugin.
//!
//! Two operating modes:
//! - **MIDI mode**: Takes MIDI input, harmonizes, outputs MIDI on per-voice channels.
//! - **Audio mode**: Takes guitar audio input, detects pitch, harmonizes, outputs MIDI.
//!
//! All harmony parameters (key, mode, scale, voices, etc.) are exposed as
//! DAW-automatable plugin parameters.

use nih_plug::prelude::*;
use std::sync::Arc;

use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig, MidiEvent as CpMidiEvent};
use contrapunk::harmony::{
    HarmonyEngine, HarmonyMode, Key, OctaveMode, ScaleMode, VoiceLeadingStyle,
};

// ── Parameter enums ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginKey {
    C,
    #[name = "C#/Db"]
    Db,
    D,
    #[name = "D#/Eb"]
    Eb,
    E,
    F,
    #[name = "F#/Gb"]
    Gb,
    G,
    #[name = "G#/Ab"]
    Ab,
    A,
    #[name = "A#/Bb"]
    Bb,
    B,
}

impl PluginKey {
    fn to_contrapunk(self) -> Key {
        match self {
            Self::C => Key::C,
            Self::Db => Key::Db,
            Self::D => Key::D,
            Self::Eb => Key::Eb,
            Self::E => Key::E,
            Self::F => Key::F,
            Self::Gb => Key::Gb,
            Self::G => Key::G,
            Self::Ab => Key::Ab,
            Self::A => Key::A,
            Self::Bb => Key::Bb,
            Self::B => Key::B,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginMode {
    #[name = "Pass Through"]
    PassThrough,
    #[name = "Diatonic 3rds"]
    DiatonicThirds,
    #[name = "Diatonic 4ths"]
    DiatonicFourths,
    #[name = "Random Below"]
    RandomBelow,
    #[name = "Random (No 2nds)"]
    RandomBelowNoSeconds,
    #[name = "Contrary Motion"]
    ContraryMotion,
    #[name = "Strict Counterpoint"]
    StrictCounterpoint,
    #[name = "Barry Harris"]
    BarryHarris,
}

impl PluginMode {
    fn to_contrapunk(self) -> HarmonyMode {
        match self {
            Self::PassThrough => HarmonyMode::PassThrough,
            Self::DiatonicThirds => HarmonyMode::DiatonicThirds,
            Self::DiatonicFourths => HarmonyMode::DiatonicFourths,
            Self::RandomBelow => HarmonyMode::RandomBelow,
            Self::RandomBelowNoSeconds => HarmonyMode::RandomBelowNoSeconds,
            Self::ContraryMotion => HarmonyMode::ContraryMotion,
            Self::StrictCounterpoint => HarmonyMode::StrictCounterpoint,
            Self::BarryHarris => HarmonyMode::BarryHarris,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginOctaveMode {
    None,
    Spread,
    #[name = "Bass/Treble Split"]
    BassTrebleSplit,
    Mirror,
}

impl PluginOctaveMode {
    fn to_contrapunk(self) -> OctaveMode {
        match self {
            Self::None => OctaveMode::None,
            Self::Spread => OctaveMode::Spread,
            Self::BassTrebleSplit => OctaveMode::BassTrebleSplit,
            Self::Mirror => OctaveMode::Mirror,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginInputMode {
    /// MIDI input — harmonize incoming MIDI notes
    #[name = "MIDI"]
    Midi,
    /// Audio input — detect pitch from guitar audio, then harmonize
    #[name = "Audio (Guitar)"]
    Audio,
}

// ── Parameters ───────────────────────────────────────────────────────

#[derive(Params)]
struct ContrapunkParams {
    #[id = "input_mode"]
    pub input_mode: EnumParam<PluginInputMode>,

    #[id = "key"]
    pub key: EnumParam<PluginKey>,

    #[id = "mode"]
    pub harmony_mode: EnumParam<PluginMode>,

    #[id = "voices"]
    pub voice_count: IntParam,

    #[id = "voice_pos"]
    pub voice_position: IntParam,

    #[id = "octave"]
    pub octave_mode: EnumParam<PluginOctaveMode>,

    #[id = "auto_key"]
    pub auto_key: BoolParam,

    #[id = "voice_lead"]
    pub voice_leading: BoolParam,
}

impl Default for ContrapunkParams {
    fn default() -> Self {
        Self {
            input_mode: EnumParam::new("Input", PluginInputMode::Midi),
            key: EnumParam::new("Key", PluginKey::C),
            harmony_mode: EnumParam::new("Mode", PluginMode::DiatonicThirds),
            voice_count: IntParam::new("Voices", 2, IntRange::Linear { min: 1, max: 4 }),
            voice_position: IntParam::new("You Play", 0, IntRange::Linear { min: 0, max: 3 }),
            octave_mode: EnumParam::new("Octave", PluginOctaveMode::None),
            auto_key: BoolParam::new("Auto Key", false),
            voice_leading: BoolParam::new("Voice Leading", false),
        }
    }
}

// ── Plugin ───────────────────────────────────────────────────────────

struct ContrapunkPlugin {
    params: Arc<ContrapunkParams>,
    engine: HarmonyEngine,
    guitar_input: Option<GuitarInput>,
    sample_rate: f32,

    // Track last param values to detect changes
    last_key: PluginKey,
    last_mode: PluginMode,
    last_octave: PluginOctaveMode,
    last_voices: i32,
    last_voice_pos: i32,
    last_auto_key: bool,
    last_voice_leading: bool,
}

impl Default for ContrapunkPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ContrapunkParams::default()),
            engine: HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds),
            guitar_input: None,
            sample_rate: 48000.0,
            last_key: PluginKey::C,
            last_mode: PluginMode::DiatonicThirds,
            last_octave: PluginOctaveMode::None,
            last_voices: 2,
            last_voice_pos: 0,
            last_auto_key: false,
            last_voice_leading: false,
        }
    }
}

impl ContrapunkPlugin {
    /// Sync DAW parameter values to the harmony engine.
    /// Only updates when values actually change.
    fn sync_params(&mut self) {
        let key = self.params.key.value();
        if key != self.last_key {
            self.engine.set_key(key.to_contrapunk());
            self.last_key = key;
        }

        let mode = self.params.harmony_mode.value();
        if mode != self.last_mode {
            self.engine.set_mode(mode.to_contrapunk());
            self.last_mode = mode;
        }

        let octave = self.params.octave_mode.value();
        if octave != self.last_octave {
            self.engine.set_octave_mode(octave.to_contrapunk());
            self.last_octave = octave;
        }

        let voices = self.params.voice_count.value();
        if voices != self.last_voices {
            self.engine.set_voice_count(voices as usize);
            self.last_voices = voices;
        }

        let vp = self.params.voice_position.value();
        if vp != self.last_voice_pos {
            self.engine.set_voice_position(vp as usize);
            self.last_voice_pos = vp;
        }

        let auto_key = self.params.auto_key.value();
        if auto_key != self.last_auto_key {
            self.engine.set_auto_key(auto_key);
            self.last_auto_key = auto_key;
        }

        let vl = self.params.voice_leading.value();
        if vl != self.last_voice_leading {
            self.engine.set_voice_leading_enabled(vl);
            self.last_voice_leading = vl;
        }
    }

    /// Send a harmonized NoteOn through the plugin's MIDI output.
    /// Melody goes on channel 1 (index 1), harmonies on channels 2-6.
    fn send_harmonized_note_on(
        &mut self,
        timing: u32,
        note: wmidi::Note,
        velocity: f32,
        context: &mut impl ProcessContext<Self>,
    ) {
        let harmonized = self.engine.harmonize_note_on(note);
        for (i, &h_note) in harmonized.iter().enumerate() {
            // MPE: Ch 2 = melody (index 1), Ch 3+ = harmony voices
            let channel = (i + 1).min(15) as u8;
            context.send_event(NoteEvent::NoteOn {
                timing,
                voice_id: None,
                channel,
                note: u8::from(h_note),
                velocity,
            });
        }
    }

    /// Send harmonized NoteOff.
    fn send_harmonized_note_off(
        &mut self,
        timing: u32,
        note: wmidi::Note,
        velocity: f32,
        context: &mut impl ProcessContext<Self>,
    ) {
        let released = self.engine.harmonize_note_off(note);
        for (i, &h_note) in released.iter().enumerate() {
            let channel = (i + 1).min(15) as u8;
            context.send_event(NoteEvent::NoteOff {
                timing,
                voice_id: None,
                channel,
                note: u8::from(h_note),
                velocity,
            });
        }
    }

    /// Process guitar audio through the pitch detection pipeline
    /// and emit MIDI events.
    fn process_audio_to_midi(
        &mut self,
        buffer: &mut Buffer,
        context: &mut impl ProcessContext<Self>,
    ) {
        let guitar = match &mut self.guitar_input {
            Some(g) => g,
            None => return,
        };

        // Feed the first channel of audio into the guitar pipeline
        let channel_data: Vec<f32> = buffer.as_slice()[0].to_vec();
        let midi_events = guitar.process_block(&channel_data);

        for event in midi_events {
            match event {
                CpMidiEvent::NoteOn { note, velocity, .. } => {
                    if let Ok(wmidi_note) = wmidi::Note::try_from(note) {
                        self.send_harmonized_note_on(
                            0,
                            wmidi_note,
                            velocity as f32 / 127.0,
                            context,
                        );
                    }
                }
                CpMidiEvent::NoteOff { note, velocity, .. } => {
                    if let Ok(wmidi_note) = wmidi::Note::try_from(note) {
                        self.send_harmonized_note_off(
                            0,
                            wmidi_note,
                            velocity as f32 / 127.0,
                            context,
                        );
                    }
                }
                CpMidiEvent::PitchBend { channel, cents } => {
                    // Convert cents to 0.0-1.0 (center = 0.5)
                    let bend_range = 48.0; // semitones
                    let normalized = 0.5 + (cents as f32 / (bend_range * 100.0));
                    context.send_event(NoteEvent::MidiPitchBend {
                        timing: 0,
                        channel: channel + 1, // offset for MPE member channels
                        value: normalized.clamp(0.0, 1.0),
                    });
                }
                CpMidiEvent::CC {
                    channel,
                    controller,
                    value,
                } => {
                    context.send_event(NoteEvent::MidiCC {
                        timing: 0,
                        channel: channel + 1,
                        cc: controller,
                        value: value as f32 / 127.0,
                    });
                }
                CpMidiEvent::ChannelPressure {
                    channel, pressure, ..
                } => {
                    context.send_event(NoteEvent::MidiChannelPressure {
                        timing: 0,
                        channel: channel + 1,
                        pressure: pressure as f32 / 127.0,
                    });
                }
                _ => {} // Ignore informational events
            }
        }
    }
}

impl Plugin for ContrapunkPlugin {
    const NAME: &'static str = "Contrapunk";
    const VENDOR: &'static str = "Contrapunk Audio";
    const URL: &'static str = "https://contrapunk.com";
    const EMAIL: &'static str = "hello@contrapunk.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        // Stereo pass-through (most common DAW configuration)
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        // Mono (guitar DI input)
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
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

        // Initialize guitar pitch detection pipeline
        let config = GuitarInputConfig {
            sample_rate: buffer_config.sample_rate as usize,
            buffer_size: 1024,
            hop_size: 256,
            ..GuitarInputConfig::default()
        };
        self.guitar_input = Some(GuitarInput::new(config));

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Sync parameter changes to engine
        self.sync_params();

        let input_mode = self.params.input_mode.value();

        match input_mode {
            PluginInputMode::Midi => {
                // MIDI-to-MIDI: harmonize incoming MIDI events
                while let Some(event) = context.next_event() {
                    match event {
                        NoteEvent::NoteOn {
                            timing,
                            note,
                            velocity,
                            ..
                        } => {
                            if let Ok(wmidi_note) = wmidi::Note::try_from(note) {
                                self.send_harmonized_note_on(timing, wmidi_note, velocity, context);
                            }
                        }
                        NoteEvent::NoteOff {
                            timing,
                            note,
                            velocity,
                            ..
                        } => {
                            if let Ok(wmidi_note) = wmidi::Note::try_from(note) {
                                self.send_harmonized_note_off(
                                    timing, wmidi_note, velocity, context,
                                );
                            }
                        }
                        // Forward other events unchanged
                        other => context.send_event(other),
                    }
                }
            }
            PluginInputMode::Audio => {
                // Audio-to-MIDI: pitch detect from guitar audio, then harmonize
                // Drain MIDI input events (ignore in audio mode)
                while context.next_event().is_some() {}

                self.process_audio_to_midi(buffer, context);
            }
        }

        // Audio passes through unchanged in both modes
        ProcessStatus::Normal
    }
}

impl ClapPlugin for ContrapunkPlugin {
    const CLAP_ID: &'static str = "com.contrapunk.harmony";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Real-time counterpoint harmony generator");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::NoteEffect,
        ClapFeature::NoteDetector,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for ContrapunkPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"ContrpnkHrm_v001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(ContrapunkPlugin);
nih_export_vst3!(ContrapunkPlugin);
