//! Contrapunk VST3/CLAP plugin.
//!
//! Two operating modes:
//! - **MIDI mode**: Takes MIDI input, harmonizes, outputs MIDI on per-voice channels.
//! - **Audio mode**: Takes guitar audio input, detects pitch, harmonizes, outputs MIDI.
//!
//! All harmony parameters (key, mode, scale, voices, etc.) are exposed as
//! DAW-automatable plugin parameters.

use nih_plug::prelude::*;
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread;
use std::time::Duration;

mod editor;
#[cfg(target_os = "macos")]
mod logic_midi;

use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig, MidiEvent as CpMidiEvent};
use contrapunk::chain::{AudioBlock, MidiBlockEvent};
use contrapunk::harmony::{HarmonyEngine, HarmonyMode, Key, OctaveMode};
use contrapunk::synth::{Synth, SynthParams};
use contrapunk_companion::{CanonLane, Companion, CounterpointLane, WorldState};
use contrapunk_transport::Transport;

const MELODY_CHANNEL: u8 = 0;
const FIRST_HARMONY_CHANNEL: u8 = 1;
const CANON_CHANNEL: u8 = 5;
const COUNTERPOINT_CHANNEL: u8 = 6;
const TRACKED_OUTPUT_NOTES: usize = 16 * 128;

fn harmony_output_channel(result_index: usize) -> u8 {
    if result_index == 0 {
        MELODY_CHANNEL
    } else {
        (FIRST_HARMONY_CHANNEL + (result_index as u8) - 1).min(CANON_CHANNEL - 1)
    }
}

fn lane_output_channel(lane: &str, fallback_channel: u8) -> u8 {
    match lane {
        "canon" => CANON_CHANNEL,
        "counterpoint" => COUNTERPOINT_CHANNEL,
        _ => fallback_channel.min(15),
    }
}

fn tracked_note_index(channel: u8, note: u8) -> usize {
    channel.min(15) as usize * 128 + note.min(127) as usize
}

fn is_all_notes_off_cc(cc: u8) -> bool {
    cc == 120 || cc == 123
}

fn combine_output_sample(existing: f32, synth: f32, has_audio_input: bool) -> f32 {
    let dry = if has_audio_input && existing.is_finite() {
        existing
    } else {
        0.0
    };
    let synth = if synth.is_finite() { synth } else { 0.0 };
    (dry + synth).clamp(-1.0, 1.0)
}

fn track_note_on(active: &mut [u32], index: usize) -> bool {
    let count = &mut active[index];
    let first_owner = *count == 0;
    *count = count.saturating_add(1);
    first_owner
}

fn track_note_off(active: &mut [u32], index: usize) -> bool {
    let count = &mut active[index];
    if *count == 0 {
        // Forward unmatched NoteOffs defensively; the upstream owner may
        // have started before this instance began tracking.
        return true;
    }
    *count -= 1;
    *count == 0
}

/// Live note tracking shared between the audio thread and the
/// editor's frame loop. The editor reads this every UI tick and
/// pushes it to the JS side as a `noteUpdate` event so the Piano /
/// Fretboard light up while the plugin is generating MIDI.
///
/// `canon_notes` and `counterpoint_notes` populate when the
/// Companion's CanonLane / CounterpointLane emit `DispatchOp`s via
/// `tick_tagged` or `on_input_tagged`. Piano colors them gold / lime.
#[derive(Clone)]
pub(crate) struct NoteCounts {
    counts: [u16; 128],
}

impl Default for NoteCounts {
    fn default() -> Self {
        Self { counts: [0; 128] }
    }
}

impl NoteCounts {
    fn insert(&mut self, note: u8) {
        let count = &mut self.counts[note.min(127) as usize];
        *count = count.saturating_add(1);
    }

    fn remove(&mut self, note: &u8) {
        let count = &mut self.counts[(*note).min(127) as usize];
        *count = count.saturating_sub(1);
    }

    fn clear(&mut self) {
        self.counts.fill(0);
    }

    #[cfg(test)]
    fn contains(&self, note: &u8) -> bool {
        self.counts[(*note).min(127) as usize] > 0
    }

    pub(crate) fn active_notes(&self) -> impl Iterator<Item = u8> + '_ {
        self.counts
            .iter()
            .enumerate()
            .filter_map(|(note, count)| (*count > 0).then_some(note as u8))
    }
}

#[derive(Default)]
pub struct PluginNoteState {
    pub(crate) input_notes: NoteCounts,
    pub(crate) harmony_notes: NoteCounts,
    pub(crate) canon_notes: NoteCounts,
    pub(crate) counterpoint_notes: NoteCounts,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct PluginGuitarSignal {
    pub(crate) rms: f32,
    pub(crate) frequency: Option<f32>,
    pub(crate) clarity: f32,
    pub(crate) note_state: u8,
    pub(crate) midi_note: u8,
}

impl PluginNoteState {
    #[cfg(test)]
    fn lane_note_on(&mut self, lane: &str, note: u8) {
        match lane {
            "canon" => {
                self.canon_notes.insert(note);
            }
            "counterpoint" => {
                self.counterpoint_notes.insert(note);
            }
            _ => {}
        }
    }

    #[cfg(test)]
    fn lane_note_off(&mut self, lane: &str, note: u8) {
        match lane {
            "canon" => {
                self.canon_notes.remove(&note);
            }
            "counterpoint" => {
                self.counterpoint_notes.remove(&note);
            }
            _ => {}
        }
    }
}

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
    #[name = "Counterpoint (Species 1)"]
    StrictCounterpoint,
    #[name = "Barry Harris (Drop-2)"]
    BarryHarris,
    #[name = "Functional Harmony"]
    FunctionalHarmony,
    #[name = "Bach Chorale (SATB)"]
    BachChorale,
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
            Self::FunctionalHarmony => HarmonyMode::FunctionalHarmony,
            Self::BachChorale => HarmonyMode::BachChorale,
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

fn effective_input_mode(configured: PluginInputMode, guitar_component: bool) -> PluginInputMode {
    if guitar_component {
        PluginInputMode::Audio
    } else {
        configured
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum PluginMidiOutputMode {
    /// Normal Contrapunk behavior: melody + generated harmony/canon/counterpoint.
    #[name = "Full Contrapunk"]
    Full,
    /// Diagnostic/utility mode: pass incoming MIDI through untouched.
    #[name = "Pass Through"]
    PassThrough,
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

    #[id = "spread"]
    pub octave_intensity: FloatParam,

    #[id = "auto_key"]
    pub auto_key: BoolParam,

    #[id = "voice_lead"]
    pub voice_leading: BoolParam,

    #[id = "synth"]
    pub synth_enabled: BoolParam,

    #[id = "midi_output"]
    pub midi_output_mode: EnumParam<PluginMidiOutputMode>,

    // v2 drops the legacy 900×700 editor state now that the production
    // workspace requires 1200×800.
    #[persist = "webview_state_v2"]
    pub webview_state: Arc<nih_plug_webview::WebViewState>,
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
            octave_intensity: FloatParam::new(
                "Spread",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0)),
            auto_key: BoolParam::new("Auto Key", false),
            voice_leading: BoolParam::new("Voice Leading", false),
            synth_enabled: BoolParam::new("Built-in Synth", true),
            midi_output_mode: EnumParam::new("MIDI Output", PluginMidiOutputMode::Full),
            webview_state: editor::default_webview_state(),
        }
    }
}

impl ContrapunkParams {
    fn with_input_mode(input_mode: PluginInputMode) -> Self {
        let mut params = Self::default();
        params.input_mode = EnumParam::new("Input", input_mode);
        params
    }
}

// ── Allocation boundary ──────────────────────────────────────────────

// ponytail: bounded RT queues fail closed with All Notes Off on overflow;
// raise these only if dense MIDI or guitar blocks prove the limits too small.
const WORKER_INPUT_CAPACITY: usize = 4096;
const WORKER_OUTPUT_CAPACITY: usize = 8192;
const WORKER_AUDIO_CAPACITY: usize = 131_072;

#[derive(Clone, Copy)]
struct WorkerParams {
    key: PluginKey,
    mode: PluginMode,
    octave: PluginOctaveMode,
    octave_intensity: f32,
    voices: i32,
    voice_position: i32,
    auto_key: bool,
    voice_leading: bool,
    sample_rate: f32,
}

impl Default for WorkerParams {
    fn default() -> Self {
        Self {
            key: PluginKey::C,
            mode: PluginMode::DiatonicThirds,
            octave: PluginOctaveMode::None,
            octave_intensity: 1.0,
            voices: 2,
            voice_position: 0,
            auto_key: false,
            voice_leading: false,
            sample_rate: 48_000.0,
        }
    }
}

#[derive(Clone, Copy)]
enum WorkerInput {
    Configure {
        generation: u64,
        params: WorkerParams,
    },
    NoteOn {
        generation: u64,
        block: u64,
        timing: u32,
        note: u8,
        velocity: f32,
    },
    NoteOff {
        generation: u64,
        block: u64,
        timing: u32,
        note: u8,
        velocity: f32,
    },
    Tick {
        generation: u64,
        block: u64,
    },
    AudioBlock {
        generation: u64,
        block: u64,
        samples: usize,
        harmonize: bool,
    },
}

#[derive(Clone, Copy)]
enum WorkerNoteSource {
    Input,
    Harmony,
    Canon,
    Counterpoint,
}

#[derive(Clone, Copy)]
enum WorkerOutput {
    NoteOn {
        generation: u64,
        block: u64,
        timing: u32,
        source: WorkerNoteSource,
        channel: u8,
        note: u8,
        velocity: f32,
    },
    NoteOff {
        generation: u64,
        block: u64,
        timing: u32,
        source: WorkerNoteSource,
        channel: u8,
        note: u8,
        velocity: f32,
    },
    PitchBend {
        generation: u64,
        channel: u8,
        value: f32,
    },
    ControlChange {
        generation: u64,
        channel: u8,
        controller: u8,
        value: f32,
    },
    ChannelPressure {
        generation: u64,
        channel: u8,
        pressure: f32,
    },
    AllNotesOff {
        generation: u64,
    },
}

impl WorkerOutput {
    fn generation(self) -> u64 {
        match self {
            Self::NoteOn { generation, .. }
            | Self::NoteOff { generation, .. }
            | Self::PitchBend { generation, .. }
            | Self::ControlChange { generation, .. }
            | Self::ChannelPressure { generation, .. }
            | Self::AllNotesOff { generation, .. } => generation,
        }
    }
}

struct MusicWorker {
    input: HeapProd<WorkerInput>,
    output: HeapCons<WorkerOutput>,
    audio: HeapProd<f32>,
    stop: Arc<AtomicBool>,
}

impl MusicWorker {
    fn new(
        engine: Arc<Mutex<HarmonyEngine>>,
        companion: Arc<Mutex<Companion>>,
        guitar_signal: Arc<Mutex<PluginGuitarSignal>>,
    ) -> Self {
        let input_rb = HeapRb::new(WORKER_INPUT_CAPACITY);
        let (input, input_rx) = input_rb.split();
        let output_rb = HeapRb::new(WORKER_OUTPUT_CAPACITY);
        let (output_tx, output) = output_rb.split();
        let audio_rb = HeapRb::new(WORKER_AUDIO_CAPACITY);
        let (audio, audio_rx) = audio_rb.split();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let handle = thread::Builder::new()
            .name("contrapunk-music".into())
            .spawn(move || {
                run_music_worker(
                    engine,
                    companion,
                    input_rx,
                    output_tx,
                    audio_rx,
                    guitar_signal,
                    worker_stop,
                )
            })
            .expect("failed to start Contrapunk music worker");
        drop(handle);

        Self {
            input,
            output,
            audio,
            stop,
        }
    }

    fn try_push(&mut self, input: WorkerInput) -> bool {
        self.input.try_push(input).is_ok()
    }

    fn try_push_audio(&mut self, samples: &[f32], command: WorkerInput) -> bool {
        // Never trade safety for seconds of pitch-detection latency if the
        // worker falls behind. Four queued blocks is the deliberate ceiling.
        if self.audio.occupied_len() > samples.len().saturating_mul(4)
            || self.input.vacant_len() == 0
            || self.audio.vacant_len() < samples.len()
        {
            return false;
        }
        if self.audio.push_slice(samples) != samples.len() {
            return false;
        }
        self.try_push(command)
    }

    fn try_pop(&mut self) -> Option<WorkerOutput> {
        self.output.try_pop()
    }
}

impl Drop for MusicWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

fn push_worker_output(
    output: &mut HeapProd<WorkerOutput>,
    stop: &AtomicBool,
    mut event: WorkerOutput,
) {
    loop {
        match output.try_push(event) {
            Ok(()) => return,
            Err(returned) => event = returned,
        }
        if stop.load(Ordering::Acquire) {
            return;
        }
        thread::yield_now();
    }
}

fn worker_note_source(lane: &str) -> WorkerNoteSource {
    match lane {
        "canon" => WorkerNoteSource::Canon,
        "counterpoint" => WorkerNoteSource::Counterpoint,
        _ => WorkerNoteSource::Harmony,
    }
}

fn push_tagged_worker_ops(
    output: &mut HeapProd<WorkerOutput>,
    stop: &AtomicBool,
    generation: u64,
    block: u64,
    timing: u32,
    tagged: &[(&'static str, contrapunk_companion::DispatchOp)],
) {
    use contrapunk_companion::DispatchOp;

    for (lane, op) in tagged {
        let source = worker_note_source(lane);
        let event = match op {
            DispatchOp::NoteOn {
                note,
                velocity,
                channel,
                ..
            } => WorkerOutput::NoteOn {
                generation,
                block,
                timing,
                source,
                channel: lane_output_channel(lane, *channel),
                note: *note,
                velocity: *velocity as f32 / 127.0,
            },
            DispatchOp::NoteOff { note, channel, .. } => WorkerOutput::NoteOff {
                generation,
                block,
                timing,
                source,
                channel: lane_output_channel(lane, *channel),
                note: *note,
                velocity: 0.0,
            },
            DispatchOp::AllNotesOff { .. } => WorkerOutput::AllNotesOff { generation },
        };
        push_worker_output(output, stop, event);
    }
}

fn push_harmony_worker_notes(
    output: &mut HeapProd<WorkerOutput>,
    stop: &AtomicBool,
    generation: u64,
    block: u64,
    timing: u32,
    velocity: f32,
    notes: &[wmidi::Note],
    note_on: bool,
) {
    for (index, note) in notes.iter().copied().enumerate() {
        let event = if note_on {
            WorkerOutput::NoteOn {
                generation,
                block,
                timing,
                source: if index == 0 {
                    WorkerNoteSource::Input
                } else {
                    WorkerNoteSource::Harmony
                },
                channel: harmony_output_channel(index),
                note: u8::from(note),
                velocity,
            }
        } else {
            WorkerOutput::NoteOff {
                generation,
                block,
                timing,
                source: if index == 0 {
                    WorkerNoteSource::Input
                } else {
                    WorkerNoteSource::Harmony
                },
                channel: harmony_output_channel(index),
                note: u8::from(note),
                velocity,
            }
        };
        push_worker_output(output, stop, event);
    }
}

fn process_worker_note(
    engine: &Arc<Mutex<HarmonyEngine>>,
    companion: &Arc<Mutex<Companion>>,
    output: &mut HeapProd<WorkerOutput>,
    stop: &AtomicBool,
    generation: u64,
    block: u64,
    timing: u32,
    note: u8,
    velocity: f32,
    note_on: bool,
    use_companion: bool,
) {
    let suppress_default = if use_companion {
        let input = if note_on {
            contrapunk_companion::InputEvent::NoteOn {
                note,
                velocity: (velocity * 127.0).round().clamp(1.0, 127.0) as u8,
                channel: 0,
            }
        } else {
            contrapunk_companion::InputEvent::NoteOff { note, channel: 0 }
        };
        let (tagged, suppress) = companion
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .on_input_tagged(input, engine);
        push_tagged_worker_ops(output, stop, generation, block, timing, &tagged);
        suppress
    } else {
        false
    };

    if suppress_default {
        return;
    }
    let Ok(note) = wmidi::Note::try_from(note) else {
        return;
    };
    let notes = if note_on {
        engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .harmonize_note_on(note)
    } else {
        engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .harmonize_note_off(note)
    };
    push_harmony_worker_notes(
        output, stop, generation, block, timing, velocity, &notes, note_on,
    );
}

fn configure_music_worker(
    engine: &Arc<Mutex<HarmonyEngine>>,
    companion: &Arc<Mutex<Companion>>,
    guitar: &mut GuitarInput,
    params: WorkerParams,
) {
    {
        let mut engine = engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        engine.set_key(params.key.to_contrapunk());
        engine.set_mode(params.mode.to_contrapunk());
        engine.set_octave_mode(params.octave.to_contrapunk());
        engine.set_octave_intensity(params.octave_intensity);
        engine.set_voice_count(params.voices as usize);
        engine.set_voice_position(params.voice_position as usize);
        engine.set_auto_key(params.auto_key);
        engine.set_voice_leading_enabled(params.voice_leading);
        engine.clear_active_notes();
    }
    companion
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .reset_runtime();
    *guitar = GuitarInput::new(GuitarInputConfig {
        sample_rate: params.sample_rate as usize,
        buffer_size: 1024,
        hop_size: 256,
        ..GuitarInputConfig::default()
    });
}

fn run_music_worker(
    engine: Arc<Mutex<HarmonyEngine>>,
    companion: Arc<Mutex<Companion>>,
    mut input: HeapCons<WorkerInput>,
    mut output: HeapProd<WorkerOutput>,
    mut audio: HeapCons<f32>,
    guitar_signal: Arc<Mutex<PluginGuitarSignal>>,
    stop: Arc<AtomicBool>,
) {
    let mut generation = 0;
    let mut guitar = GuitarInput::new(GuitarInputConfig::default());
    let mut audio_block = Vec::new();

    while !stop.load(Ordering::Acquire) {
        let mut did_work = false;
        while let Some(command) = input.try_pop() {
            did_work = true;
            match command {
                WorkerInput::Configure {
                    generation: next_generation,
                    params,
                } => {
                    generation = next_generation;
                    configure_music_worker(&engine, &companion, &mut guitar, params);
                }
                WorkerInput::NoteOn {
                    generation: event_generation,
                    block,
                    timing,
                    note,
                    velocity,
                } if event_generation == generation => process_worker_note(
                    &engine,
                    &companion,
                    &mut output,
                    &stop,
                    generation,
                    block,
                    timing,
                    note,
                    velocity,
                    true,
                    true,
                ),
                WorkerInput::NoteOff {
                    generation: event_generation,
                    block,
                    timing,
                    note,
                    velocity,
                } if event_generation == generation => process_worker_note(
                    &engine,
                    &companion,
                    &mut output,
                    &stop,
                    generation,
                    block,
                    timing,
                    note,
                    velocity,
                    false,
                    true,
                ),
                WorkerInput::Tick {
                    generation: event_generation,
                    block,
                } if event_generation == generation => {
                    let tagged = companion
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .tick_tagged(&engine);
                    push_tagged_worker_ops(&mut output, &stop, generation, block, 0, &tagged);
                }
                WorkerInput::AudioBlock {
                    generation: event_generation,
                    block,
                    samples,
                    harmonize,
                } if event_generation == generation => {
                    audio_block.resize(samples, 0.0);
                    let mut read = 0;
                    while read < samples && !stop.load(Ordering::Acquire) {
                        let count = audio.pop_slice(&mut audio_block[read..]);
                        if count == 0 {
                            thread::yield_now();
                        }
                        read += count;
                    }
                    if read != samples {
                        continue;
                    }
                    let events = guitar.process_block(&audio_block);
                    let (frequency, clarity) = guitar
                        .last_debug_pitch
                        .map(|(frequency, clarity)| (Some(frequency), clarity))
                        .unwrap_or((None, 0.0));
                    *guitar_signal
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) = PluginGuitarSignal {
                        rms: guitar.prev_rms(),
                        frequency,
                        clarity,
                        note_state: guitar.note_state_name(),
                        midi_note: guitar
                            .current_note()
                            .map(|note| note.midi_note)
                            .unwrap_or(0),
                    };
                    for event in events {
                        match event {
                            CpMidiEvent::NoteOn { note, velocity, .. } if harmonize => {
                                process_worker_note(
                                    &engine,
                                    &companion,
                                    &mut output,
                                    &stop,
                                    generation,
                                    block,
                                    0,
                                    note,
                                    velocity as f32 / 127.0,
                                    true,
                                    false,
                                );
                            }
                            CpMidiEvent::NoteOff { note, velocity, .. } if harmonize => {
                                process_worker_note(
                                    &engine,
                                    &companion,
                                    &mut output,
                                    &stop,
                                    generation,
                                    block,
                                    0,
                                    note,
                                    velocity as f32 / 127.0,
                                    false,
                                    false,
                                );
                            }
                            CpMidiEvent::NoteOn { note, velocity, .. } => push_worker_output(
                                &mut output,
                                &stop,
                                WorkerOutput::NoteOn {
                                    generation,
                                    block,
                                    timing: 0,
                                    source: WorkerNoteSource::Input,
                                    channel: MELODY_CHANNEL,
                                    note,
                                    velocity: velocity as f32 / 127.0,
                                },
                            ),
                            CpMidiEvent::NoteOff { note, velocity, .. } => push_worker_output(
                                &mut output,
                                &stop,
                                WorkerOutput::NoteOff {
                                    generation,
                                    block,
                                    timing: 0,
                                    source: WorkerNoteSource::Input,
                                    channel: MELODY_CHANNEL,
                                    note,
                                    velocity: velocity as f32 / 127.0,
                                },
                            ),
                            CpMidiEvent::PitchBend { channel, cents } => push_worker_output(
                                &mut output,
                                &stop,
                                WorkerOutput::PitchBend {
                                    generation,
                                    channel: channel.min(15),
                                    value: (0.5 + cents as f32 / 4_800.0).clamp(0.0, 1.0),
                                },
                            ),
                            CpMidiEvent::CC {
                                channel,
                                controller,
                                value,
                            } => push_worker_output(
                                &mut output,
                                &stop,
                                WorkerOutput::ControlChange {
                                    generation,
                                    channel: channel.min(15),
                                    controller,
                                    value: value as f32 / 127.0,
                                },
                            ),
                            CpMidiEvent::ChannelPressure {
                                channel, pressure, ..
                            } => push_worker_output(
                                &mut output,
                                &stop,
                                WorkerOutput::ChannelPressure {
                                    generation,
                                    channel: channel.min(15),
                                    pressure: pressure as f32 / 127.0,
                                },
                            ),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        if !did_work {
            thread::park_timeout(Duration::from_millis(1));
        }
    }
}

// ── Plugin ───────────────────────────────────────────────────────────

struct ContrapunkPlugin {
    params: Arc<ContrapunkParams>,
    /// Sample-driven transport. Synced from `ProcessContext::transport`
    /// at the top of every `process()` call so canon lane scheduling
    /// follows the DAW's master clock.
    transport: Arc<Transport>,
    /// Companion orchestrator owning the canon + counterpoint lanes.
    /// Shared by the editor and non-real-time music worker; neither lock
    /// is ever taken by the audio callback.
    companion: Arc<Mutex<Companion>>,
    worker: MusicWorker,
    #[cfg(target_os = "macos")]
    logic_midi: logic_midi::LogicMidiOutput,
    guitar_component: bool,
    worker_params: WorkerParams,
    worker_generation: u64,
    worker_block: u64,
    worker_config_pending: bool,
    sample_rate: f32,
    has_audio_input: bool,
    synth: Synth,
    synth_params: Arc<SynthParams>,
    synth_scratch: Vec<f32>,

    /// Shared with the editor for noteUpdate emission. The audio thread
    /// updates it while draining worker output; the editor reads each tick.
    note_state: Arc<Mutex<PluginNoteState>>,
    /// Latest detector frame, written by the music worker and read by the editor.
    guitar_signal: Arc<Mutex<PluginGuitarSignal>>,

    /// UI-triggered panic. The editor sets this atomically; the audio thread
    /// emits actual MIDI note-offs/CCs from process(), where host MIDI output is legal.
    panic_requested: Arc<AtomicBool>,

    /// Notes sent out to the DAW, counted by (channel, note). Used for hard
    /// all-notes-off on host stop and panic so downstream synths don't hang.
    active_output_notes: [u32; TRACKED_OUTPUT_NOTES],
    /// Built-in synth ownership is pitch-only; MIDI channels do not
    /// create independent synth voices.
    active_synth_notes: [u32; 128],

    /// A failed best-effort UI-state clear is retried at the start of
    /// the next audio block. The audio thread never waits for the editor.
    note_state_clear_pending: bool,

    // Track last param values to detect changes
    last_key: PluginKey,
    last_mode: PluginMode,
    last_octave: PluginOctaveMode,
    last_octave_intensity: f32,
    last_voices: i32,
    last_voice_pos: i32,
    last_auto_key: bool,
    last_voice_leading: bool,
    last_input_mode: PluginInputMode,
    last_midi_output_mode: PluginMidiOutputMode,
}

impl Default for ContrapunkPlugin {
    fn default() -> Self {
        let guitar_component = {
            #[cfg(target_os = "macos")]
            {
                logic_midi::loaded_from_guitar_component()
            }
            #[cfg(not(target_os = "macos"))]
            {
                false
            }
        };
        let default_input_mode = if guitar_component {
            PluginInputMode::Audio
        } else {
            PluginInputMode::Midi
        };

        // Build engine, transport, and Companion in the same order as
        // Tauri. The music worker owns all allocating engine mutation.
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::DiatonicThirds,
        )));
        let transport = Transport::new(48_000);
        let world = WorldState::new(Arc::clone(&transport), Arc::clone(&engine));
        let companion = Arc::new(Mutex::new(Companion::new(world)));
        let synth_params = Arc::new(SynthParams::default());
        let (_synth_tx, synth_rx) = mpsc::channel();
        let synth = Synth::new(Arc::clone(&synth_params), synth_rx, 48_000);
        {
            let mut c = companion
                .lock()
                .expect("companion mutex poisoned at plugin init");
            c.lanes.push(Box::new(CanonLane::new()));
            c.lanes.push(Box::new(CounterpointLane::new()));
        }
        let guitar_signal = Arc::new(Mutex::new(PluginGuitarSignal::default()));
        let worker = MusicWorker::new(
            Arc::clone(&engine),
            Arc::clone(&companion),
            Arc::clone(&guitar_signal),
        );
        Self {
            params: Arc::new(ContrapunkParams::with_input_mode(default_input_mode)),
            transport,
            companion,
            worker,
            #[cfg(target_os = "macos")]
            logic_midi: logic_midi::LogicMidiOutput::new(),
            guitar_component,
            worker_params: WorkerParams::default(),
            worker_generation: 1,
            worker_block: 0,
            worker_config_pending: true,
            sample_rate: 48000.0,
            has_audio_input: false,
            synth,
            synth_params,
            synth_scratch: Vec::new(),
            note_state: Arc::new(Mutex::new(PluginNoteState::default())),
            guitar_signal,
            panic_requested: Arc::new(AtomicBool::new(false)),
            active_output_notes: [0; TRACKED_OUTPUT_NOTES],
            active_synth_notes: [0; 128],
            note_state_clear_pending: false,
            last_key: PluginKey::C,
            last_mode: PluginMode::DiatonicThirds,
            last_octave: PluginOctaveMode::None,
            last_octave_intensity: 1.0,
            last_voices: 2,
            last_voice_pos: 0,
            last_auto_key: false,
            last_voice_leading: false,
            last_input_mode: default_input_mode,
            last_midi_output_mode: PluginMidiOutputMode::Full,
        }
    }
}

impl ContrapunkPlugin {
    fn effective_input_mode(&self) -> PluginInputMode {
        effective_input_mode(self.params.input_mode.value(), self.guitar_component)
    }

    /// Snapshot DAW parameters for the non-real-time music worker.
    /// The audio callback only copies scalars and writes a bounded queue.
    fn sync_params(&mut self) -> bool {
        self.synth_params
            .set_enabled(self.params.synth_enabled.value());

        let params = WorkerParams {
            key: self.params.key.value(),
            mode: self.params.harmony_mode.value(),
            octave: self.params.octave_mode.value(),
            octave_intensity: self.params.octave_intensity.value(),
            voices: self.params.voice_count.value(),
            voice_position: self.params.voice_position.value(),
            auto_key: self.params.auto_key.value(),
            voice_leading: self.params.voice_leading.value(),
            sample_rate: self.sample_rate,
        };
        if params.key == self.last_key
            && params.mode == self.last_mode
            && params.octave == self.last_octave
            && (params.octave_intensity - self.last_octave_intensity).abs() < f32::EPSILON
            && params.voices == self.last_voices
            && params.voice_position == self.last_voice_pos
            && params.auto_key == self.last_auto_key
            && params.voice_leading == self.last_voice_leading
            && (params.sample_rate - self.worker_params.sample_rate).abs() < f32::EPSILON
        {
            return false;
        }

        self.worker_params = params;
        self.last_key = params.key;
        self.last_mode = params.mode;
        self.last_octave = params.octave;
        self.last_octave_intensity = params.octave_intensity;
        self.last_voices = params.voices;
        self.last_voice_pos = params.voice_position;
        self.last_auto_key = params.auto_key;
        self.last_voice_leading = params.voice_leading;
        true
    }

    fn invalidate_worker(&mut self) {
        self.worker_generation = self.worker_generation.wrapping_add(1);
        self.worker_config_pending = true;
    }

    fn flush_worker_config(&mut self) -> bool {
        if !self.worker_config_pending {
            return true;
        }
        if self.worker.try_push(WorkerInput::Configure {
            generation: self.worker_generation,
            params: self.worker_params,
        }) {
            self.worker_config_pending = false;
            true
        } else {
            false
        }
    }

    fn synth_note_on(&mut self, note: u8, velocity: f32) {
        let velocity = (velocity * 127.0).round().clamp(1.0, 127.0) as u8;
        self.synth
            .midi_event(MidiBlockEvent::NoteOn { note, velocity });
    }

    fn synth_note_off(&mut self, note: u8) {
        self.synth.midi_event(MidiBlockEvent::NoteOff { note });
    }

    fn synth_all_notes_off(&mut self) {
        self.synth.midi_event(MidiBlockEvent::AllNotesOff);
    }

    fn emit_note_on(
        &mut self,
        timing: u32,
        channel: u8,
        note: u8,
        velocity: f32,
        context: &mut impl ProcessContext<Self>,
    ) {
        let channel = channel.min(15);
        if track_note_on(
            &mut self.active_output_notes,
            tracked_note_index(channel, note),
        ) {
            context.send_event(NoteEvent::NoteOn {
                timing,
                voice_id: None,
                channel,
                note,
                velocity,
            });
            #[cfg(target_os = "macos")]
            if self.effective_input_mode() == PluginInputMode::Audio {
                self.logic_midi.note_on(channel, note, velocity);
            }
        }
        if track_note_on(&mut self.active_synth_notes, note as usize) {
            self.synth_note_on(note, velocity);
        }
    }

    fn emit_note_off(
        &mut self,
        timing: u32,
        channel: u8,
        note: u8,
        velocity: f32,
        context: &mut impl ProcessContext<Self>,
    ) {
        let channel = channel.min(15);
        if track_note_off(
            &mut self.active_output_notes,
            tracked_note_index(channel, note),
        ) {
            context.send_event(NoteEvent::NoteOff {
                timing,
                voice_id: None,
                channel,
                note,
                velocity,
            });
            #[cfg(target_os = "macos")]
            if self.effective_input_mode() == PluginInputMode::Audio {
                self.logic_midi.note_off(channel, note, velocity);
            }
        }
        if track_note_off(&mut self.active_synth_notes, note as usize) {
            self.synth_note_off(note);
        }
    }

    fn hard_all_notes_off(&mut self, timing: u32, context: &mut impl ProcessContext<Self>) {
        for channel in 0u8..16 {
            for note in 0u8..128 {
                let count = &mut self.active_output_notes[tracked_note_index(channel, note)];
                if *count == 0 {
                    continue;
                }
                *count = 0;
                context.send_event(NoteEvent::NoteOff {
                    timing,
                    voice_id: None,
                    channel,
                    note,
                    velocity: 0.0,
                });
            }
            context.send_event(NoteEvent::MidiCC {
                timing,
                channel,
                cc: 120, // All Sound Off
                value: 0.0,
            });
            context.send_event(NoteEvent::MidiCC {
                timing,
                channel,
                cc: 123, // All Notes Off
                value: 0.0,
            });
        }
        #[cfg(target_os = "macos")]
        if self.effective_input_mode() == PluginInputMode::Audio {
            self.logic_midi.all_notes_off();
        }
        self.invalidate_worker();
        self.clear_note_state();
    }

    fn render_builtin_synth(&mut self, buffer: &mut Buffer) {
        let channels = buffer.channels();
        let samples = buffer.samples();
        if channels == 0 || samples == 0 {
            return;
        }

        let len = samples * channels;
        if len > self.synth_scratch.len() {
            // A host must not receive stale/uninitialized output when it
            // supplies a block larger than the negotiated maximum.
            if !self.has_audio_input {
                for channel in buffer.as_slice() {
                    channel.fill(0.0);
                }
            }
            return;
        }

        let scratch = &mut self.synth_scratch[..len];
        self.synth.render(scratch, channels);

        let output = buffer.as_slice();
        for frame in 0..samples {
            let base = frame * channels;
            for ch in 0..channels {
                output[ch][frame] = combine_output_sample(
                    output[ch][frame],
                    scratch[base + ch],
                    self.has_audio_input,
                );
            }
        }
    }

    fn try_clear_note_state(&mut self) {
        let cleared = if let Ok(mut s) = self.note_state.try_lock() {
            s.input_notes.clear();
            s.harmony_notes.clear();
            s.canon_notes.clear();
            s.counterpoint_notes.clear();
            true
        } else {
            false
        };
        self.note_state_clear_pending = !cleared;
    }

    fn clear_note_state(&mut self) {
        self.active_synth_notes.fill(0);
        self.synth_all_notes_off();
        self.try_clear_note_state();
    }

    /// Mirror nih-plug's host-provided `Transport` snapshot into our
    /// sample-driven `contrapunk_transport::Transport`. Reads bpm /
    /// play-state / pos_samples once per `process()` call. Loop and
    /// locate events surface as a `pos_samples` jump bigger than a
    /// few buffers — re-seat sample_pos via `set_sample_pos` so
    /// beat-crossings stay aligned. Lock-free atomic writes only.
    fn sync_dawtransport(&mut self, context: &mut impl ProcessContext<Self>) {
        let dt = context.transport();
        let tempo = dt.tempo;
        let time_sig_numerator = dt.time_sig_numerator;
        let time_sig_denominator = dt.time_sig_denominator;
        let playing = dt.playing;
        let pos_samples = dt.pos_samples();
        if let Some(tempo) = tempo {
            self.transport.set_bpm(tempo);
        }
        if let (Some(num), Some(_den)) = (time_sig_numerator, time_sig_denominator) {
            // Contrapunk's Transport stores numerator + beat-unit
            // separately; nih-plug uses i32 for both, we clamp.
            self.transport
                .set_time_signature(num.max(1) as u8, _den.max(1) as u8);
        }
        match (playing, self.transport.is_running()) {
            (true, false) => self.transport.play(),
            (false, true) => {
                self.transport.stop();
                self.hard_all_notes_off(0, context);
            }
            _ => {}
        }
        if let Some(host_pos) = pos_samples {
            let host_pos = host_pos.max(0) as u64;
            let our_pos = self.transport.sample_pos();
            let delta = host_pos as i64 - our_pos as i64;
            // Threshold: anything beyond ~4 typical buffers (256 *
            // 4 = 1024 samples) is a discontinuity, not natural
            // drift. Re-seat to host position so canon scheduling
            // doesn't lag through a loop wrap.
            if delta < 0 || delta.unsigned_abs() > 1024 {
                self.transport.set_sample_pos(host_pos);
            }
        }
    }

    fn update_worker_note_state(&mut self, source: WorkerNoteSource, note: u8, note_on: bool) {
        let Ok(mut state) = self.note_state.try_lock() else {
            return;
        };
        let notes = match source {
            WorkerNoteSource::Input => &mut state.input_notes,
            WorkerNoteSource::Harmony => &mut state.harmony_notes,
            WorkerNoteSource::Canon => &mut state.canon_notes,
            WorkerNoteSource::Counterpoint => &mut state.counterpoint_notes,
        };
        if note_on {
            notes.insert(note);
        } else {
            notes.remove(&note);
        }
    }

    fn drain_worker_outputs(&mut self, frames: usize, context: &mut impl ProcessContext<Self>) {
        while let Some(event) = self.worker.try_pop() {
            if event.generation() != self.worker_generation {
                continue;
            }
            match event {
                WorkerOutput::NoteOn {
                    block,
                    timing,
                    source,
                    channel,
                    note,
                    velocity,
                    ..
                } => {
                    let timing = if block == self.worker_block {
                        timing.min(frames.saturating_sub(1) as u32)
                    } else {
                        0
                    };
                    self.emit_note_on(timing, channel, note, velocity, context);
                    self.update_worker_note_state(source, note, true);
                }
                WorkerOutput::NoteOff {
                    block,
                    timing,
                    source,
                    channel,
                    note,
                    velocity,
                    ..
                } => {
                    let timing = if block == self.worker_block {
                        timing.min(frames.saturating_sub(1) as u32)
                    } else {
                        0
                    };
                    self.emit_note_off(timing, channel, note, velocity, context);
                    self.update_worker_note_state(source, note, false);
                }
                WorkerOutput::PitchBend { channel, value, .. } => {
                    context.send_event(NoteEvent::MidiPitchBend {
                        timing: 0,
                        channel,
                        value,
                    });
                    #[cfg(target_os = "macos")]
                    if self.effective_input_mode() == PluginInputMode::Audio {
                        self.logic_midi.pitch_bend(channel, value);
                    }
                }
                WorkerOutput::ControlChange {
                    channel,
                    controller,
                    value,
                    ..
                } => {
                    context.send_event(NoteEvent::MidiCC {
                        timing: 0,
                        channel,
                        cc: controller,
                        value,
                    });
                    #[cfg(target_os = "macos")]
                    if self.effective_input_mode() == PluginInputMode::Audio {
                        self.logic_midi.control_change(channel, controller, value);
                    }
                }
                WorkerOutput::ChannelPressure {
                    channel, pressure, ..
                } => {
                    context.send_event(NoteEvent::MidiChannelPressure {
                        timing: 0,
                        channel,
                        pressure,
                    });
                    #[cfg(target_os = "macos")]
                    if self.effective_input_mode() == PluginInputMode::Audio {
                        self.logic_midi.channel_pressure(channel, pressure);
                    }
                }
                WorkerOutput::AllNotesOff { .. } => self.hard_all_notes_off(0, context),
            }
        }
    }

    fn try_enqueue_worker_note(
        &mut self,
        timing: u32,
        note: u8,
        velocity: f32,
        note_on: bool,
    ) -> bool {
        let input = if note_on {
            WorkerInput::NoteOn {
                generation: self.worker_generation,
                block: self.worker_block,
                timing,
                note,
                velocity,
            }
        } else {
            WorkerInput::NoteOff {
                generation: self.worker_generation,
                block: self.worker_block,
                timing,
                note,
                velocity,
            }
        };
        self.worker.try_push(input)
    }

    fn process_midi_passthrough(&mut self, context: &mut impl ProcessContext<Self>) {
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    timing,
                    channel,
                    note,
                    velocity,
                    ..
                } => {
                    if let Ok(mut s) = self.note_state.try_lock() {
                        s.input_notes.insert(note);
                    }
                    self.emit_note_on(timing, channel, note, velocity, context);
                }
                NoteEvent::NoteOff {
                    timing,
                    channel,
                    note,
                    velocity,
                    ..
                } => {
                    if let Ok(mut s) = self.note_state.try_lock() {
                        s.input_notes.remove(&note);
                    }
                    self.emit_note_off(timing, channel, note, velocity, context);
                }
                NoteEvent::Choke {
                    timing,
                    channel,
                    note,
                    ..
                } => {
                    if let Ok(mut s) = self.note_state.try_lock() {
                        s.input_notes.remove(&note);
                    }
                    self.emit_note_off(timing, channel, note, 0.0, context);
                }
                other @ NoteEvent::MidiCC { timing, cc, .. } if is_all_notes_off_cc(cc) => {
                    self.hard_all_notes_off(timing, context);
                    context.send_event(other);
                }
                other => context.send_event(other),
            }
        }
    }

    fn try_enqueue_audio(&mut self, buffer: &mut Buffer, harmonize: bool) -> bool {
        if buffer.channels() == 0 {
            return true;
        }
        let samples = &buffer.as_slice()[0];
        self.worker.try_push_audio(
            samples,
            WorkerInput::AudioBlock {
                generation: self.worker_generation,
                block: self.worker_block,
                samples: samples.len(),
                harmonize,
            },
        )
    }
}

impl Plugin for ContrapunkPlugin {
    const NAME: &'static str = "Contrapunk";
    const VENDOR: &'static str = "Contrapunk Audio";
    const URL: &'static str = "https://contrapunk.com";
    const EMAIL: &'static str = "hello@contrapunk.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        // Instrument mode: MIDI in, built-in synth out.
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        // Stereo pass-through + built-in synth (for guitar/audio tracks).
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        #[cfg(feature = "au-generic-ui")]
        {
            None
        }

        #[cfg(not(feature = "au-generic-ui"))]
        {
            Some(Box::new(editor::create_editor(
                self.params.clone(),
                Arc::clone(&self.note_state),
                Arc::clone(&self.guitar_signal),
                Arc::clone(&self.companion),
                Arc::clone(&self.panic_requested),
                self.guitar_component,
                &self.params.webview_state,
            )))
        }
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.has_audio_input = audio_io_layout.main_input_channels.is_some();
        self.synth.set_sample_rate(buffer_config.sample_rate as u32);
        let channels = audio_io_layout
            .main_output_channels
            .map(|c| c.get() as usize)
            .unwrap_or(2)
            .max(1);
        self.synth_scratch
            .resize(buffer_config.max_buffer_size as usize * channels, 0.0);

        self.worker_params.sample_rate = buffer_config.sample_rate;
        self.invalidate_worker();
        true
    }

    fn reset(&mut self) {
        self.synth.reset();
        #[cfg(target_os = "macos")]
        self.logic_midi.all_notes_off();
        self.active_output_notes.fill(0);
        self.active_synth_notes.fill(0);
        self.invalidate_worker();
        self.clear_note_state();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.worker_block = self.worker_block.wrapping_add(1);
        let frames = buffer.samples();

        if self.sync_params() {
            self.hard_all_notes_off(0, context);
        }
        if self.note_state_clear_pending {
            self.try_clear_note_state();
        }

        self.sync_dawtransport(context);
        self.transport.advance(frames as u32);

        if self.panic_requested.swap(false, Ordering::AcqRel) {
            self.hard_all_notes_off(0, context);
        }

        let output_mode = self.params.midi_output_mode.value();
        if output_mode != self.last_midi_output_mode {
            self.hard_all_notes_off(0, context);
            self.last_midi_output_mode = output_mode;
        }

        let input_mode = self.effective_input_mode();
        if input_mode != self.last_input_mode {
            self.hard_all_notes_off(0, context);
            self.last_input_mode = input_mode;
        }

        let worker_ready = self.flush_worker_config();
        self.drain_worker_outputs(frames, context);

        match input_mode {
            PluginInputMode::Midi if output_mode == PluginMidiOutputMode::PassThrough => {
                self.process_midi_passthrough(context);
            }
            PluginInputMode::Midi => {
                let mut queue_ok = worker_ready;
                let mut queue_overflow = false;
                while let Some(event) = context.next_event() {
                    match event {
                        NoteEvent::NoteOn {
                            timing,
                            note,
                            velocity,
                            ..
                        } if queue_ok => {
                            queue_ok = self.try_enqueue_worker_note(timing, note, velocity, true);
                            queue_overflow |= !queue_ok;
                        }
                        NoteEvent::NoteOff {
                            timing,
                            note,
                            velocity,
                            ..
                        } if queue_ok => {
                            queue_ok = self.try_enqueue_worker_note(timing, note, velocity, false);
                            queue_overflow |= !queue_ok;
                        }
                        NoteEvent::Choke { timing, note, .. } if queue_ok => {
                            queue_ok = self.try_enqueue_worker_note(timing, note, 0.0, false);
                            queue_overflow |= !queue_ok;
                        }
                        other @ NoteEvent::MidiCC { timing, cc, .. } if is_all_notes_off_cc(cc) => {
                            self.hard_all_notes_off(timing, context);
                            context.send_event(other);
                            queue_ok = false;
                        }
                        NoteEvent::NoteOn { .. }
                        | NoteEvent::NoteOff { .. }
                        | NoteEvent::Choke { .. } => queue_ok = false,
                        other => context.send_event(other),
                    }
                }

                if queue_ok
                    && !self.worker.try_push(WorkerInput::Tick {
                        generation: self.worker_generation,
                        block: self.worker_block,
                    })
                {
                    queue_overflow = true;
                }
                if queue_overflow {
                    self.hard_all_notes_off(0, context);
                }
            }
            PluginInputMode::Audio => {
                while context.next_event().is_some() {}
                if worker_ready
                    && !self.try_enqueue_audio(buffer, output_mode == PluginMidiOutputMode::Full)
                {
                    self.hard_all_notes_off(0, context);
                }
            }
        }

        // Fast workers can return within the same block. Otherwise output
        // arrives at the next block start, avoiding any audio-thread wait.
        self.drain_worker_outputs(frames, context);
        self.render_builtin_synth(buffer);

        ProcessStatus::Normal
    }
}

impl ClapPlugin for ContrapunkPlugin {
    const CLAP_ID: &'static str = "com.contrapunk.harmony";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Real-time counterpoint harmony generator");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::NoteEffect,
        ClapFeature::NoteDetector,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for ContrapunkPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"ContrpnkHrm_v001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(ContrapunkPlugin);
nih_export_vst3!(ContrapunkPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guitar_component_always_uses_audio_input() {
        assert_eq!(
            effective_input_mode(PluginInputMode::Midi, true),
            PluginInputMode::Audio
        );
        assert_eq!(
            effective_input_mode(PluginInputMode::Midi, false),
            PluginInputMode::Midi
        );
    }

    #[test]
    fn plugin_channel_map_is_stable() {
        assert_eq!(harmony_output_channel(0), 0);
        assert_eq!(harmony_output_channel(1), 1);
        assert_eq!(harmony_output_channel(4), 4);
        assert_eq!(lane_output_channel("canon", 0), 5);
        assert_eq!(lane_output_channel("counterpoint", 0), 6);
    }

    #[test]
    fn output_note_tracking_counts_overlaps_without_allocation() {
        let mut active = [0; TRACKED_OUTPUT_NOTES];
        let first = tracked_note_index(5, 64);
        let second = tracked_note_index(6, 67);
        assert!(track_note_on(&mut active, first));
        assert!(!track_note_on(&mut active, first));
        assert!(track_note_on(&mut active, second));
        assert!(!track_note_off(&mut active, first));
        assert!(track_note_off(&mut active, first));

        assert_eq!(active[first], 0);
        assert_eq!(active[second], 1);
    }

    #[test]
    fn sustain_release_is_not_global_panic() {
        assert!(!is_all_notes_off_cc(64));
        assert!(is_all_notes_off_cc(120));
        assert!(is_all_notes_off_cc(123));
    }

    #[test]
    fn instrument_output_replaces_host_buffer_instead_of_mixing_garbage() {
        assert_eq!(combine_output_sample(0.75, 0.25, false), 0.25);
        assert_eq!(combine_output_sample(f32::NAN, 0.0, false), 0.0);
        assert_eq!(combine_output_sample(0.25, 0.5, true), 0.75);
    }

    #[test]
    fn lane_note_state_does_not_remove_same_pitch_harmony() {
        let mut state = PluginNoteState::default();
        state.harmony_notes.insert(64);
        state.lane_note_on("canon", 64);
        state.lane_note_off("canon", 64);

        assert!(state.harmony_notes.contains(&64));
        assert!(!state.canon_notes.contains(&64));
    }

    #[test]
    fn music_worker_publishes_guitar_signal() {
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::DiatonicThirds,
        )));
        let transport = Transport::new(48_000);
        let companion = Arc::new(Mutex::new(Companion::new(WorldState::new(
            transport,
            Arc::clone(&engine),
        ))));
        let signal = Arc::new(Mutex::new(PluginGuitarSignal::default()));
        let mut worker = MusicWorker::new(engine, companion, Arc::clone(&signal));
        assert!(worker.try_push(WorkerInput::Configure {
            generation: 9,
            params: WorkerParams::default(),
        }));
        let samples: Vec<f32> = (0..4096)
            .map(|sample| (sample as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin() * 0.2)
            .collect();
        assert!(worker.try_push_audio(
            &samples,
            WorkerInput::AudioBlock {
                generation: 9,
                block: 1,
                samples: samples.len(),
                harmonize: true,
            },
        ));

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            if signal
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .rms
                > 0.0
            {
                return;
            }
            thread::yield_now();
        }
        panic!("music worker did not publish guitar RMS");
    }

    #[test]
    fn music_worker_returns_harmony_without_audio_thread_engine_access() {
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::DiatonicThirds,
        )));
        let transport = Transport::new(48_000);
        let companion = Arc::new(Mutex::new(Companion::new(WorldState::new(
            transport,
            Arc::clone(&engine),
        ))));
        let mut worker = MusicWorker::new(
            engine,
            companion,
            Arc::new(Mutex::new(PluginGuitarSignal::default())),
        );
        assert!(worker.try_push(WorkerInput::Configure {
            generation: 7,
            params: WorkerParams::default(),
        }));
        assert!(worker.try_push(WorkerInput::NoteOn {
            generation: 6,
            block: 1,
            timing: 0,
            note: 61,
            velocity: 1.0,
        }));
        assert!(worker.try_push(WorkerInput::NoteOn {
            generation: 7,
            block: 1,
            timing: 12,
            note: 60,
            velocity: 1.0,
        }));

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut notes = [false; 128];
        while std::time::Instant::now() < deadline {
            if let Some(WorkerOutput::NoteOn { note, .. }) = worker.try_pop() {
                notes[note as usize] = true;
                if notes[60] && notes[57] {
                    break;
                }
            } else {
                thread::yield_now();
            }
        }
        assert!(notes[60], "worker did not return the player note");
        assert!(notes[57], "worker did not return the diatonic third below");
        assert!(!notes[61], "worker emitted a stale generation");

        assert!(worker.try_push(WorkerInput::NoteOff {
            generation: 7,
            block: 2,
            timing: 0,
            note: 60,
            velocity: 0.0,
        }));
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut released = [false; 128];
        while std::time::Instant::now() < deadline {
            if let Some(WorkerOutput::NoteOff { note, .. }) = worker.try_pop() {
                released[note as usize] = true;
                if released[60] && released[57] {
                    break;
                }
            } else {
                thread::yield_now();
            }
        }
        assert!(released[60] && released[57], "worker lost note ownership");
    }
}
