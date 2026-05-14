//! Contrapunk VST3/CLAP plugin.
//!
//! Two operating modes:
//! - **MIDI mode**: Takes MIDI input, harmonizes, outputs MIDI on per-voice channels.
//! - **Audio mode**: Takes guitar audio input, detects pitch, harmonizes, outputs MIDI.
//!
//! All harmony parameters (key, mode, scale, voices, etc.) are exposed as
//! DAW-automatable plugin parameters.

use nih_plug::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

mod editor;

use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig, MidiEvent as CpMidiEvent};
use contrapunk::harmony::{
    HarmonyEngine, HarmonyMode, Key, OctaveMode, ScaleMode, VoiceLeadingStyle,
};
use contrapunk_companion::{CanonLane, Companion, CounterpointLane, WorldState};
use contrapunk_transport::Transport;

/// Live note tracking shared between the audio thread and the
/// editor's frame loop. The editor reads this every UI tick and
/// pushes it to the JS side as a `noteUpdate` event so the Piano /
/// Fretboard light up while the plugin is generating MIDI.
///
/// `canon_notes` and `counterpoint_notes` populate when the
/// Companion's CanonLane / CounterpointLane emit `DispatchOp`s via
/// `tick_tagged` or `on_input_tagged`. Piano colors them gold / lime.
#[derive(Default)]
pub struct PluginNoteState {
    pub input_notes: HashSet<u8>,
    pub harmony_notes: HashSet<u8>,
    pub canon_notes: HashSet<u8>,
    pub counterpoint_notes: HashSet<u8>,
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

    #[persist = "webview_state"]
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
            auto_key: BoolParam::new("Auto Key", false),
            voice_leading: BoolParam::new("Voice Leading", false),
            webview_state: editor::default_webview_state(),
        }
    }
}

// ── Plugin ───────────────────────────────────────────────────────────

struct ContrapunkPlugin {
    params: Arc<ContrapunkParams>,
    /// Engine wrapped in `Arc<Mutex<...>>` so the Companion's
    /// `WorldState` (which expects `Arc<Mutex<HarmonyEngine>>`) and
    /// the editor's IPC handlers (which need to read/write through
    /// the same engine instance the audio thread sees) can share it.
    /// Audio thread holds the lock briefly per block; editor uses
    /// `try_lock` on the companion side to avoid blocking the
    /// rendering callback.
    engine: Arc<Mutex<HarmonyEngine>>,
    /// Sample-driven transport. Synced from `ProcessContext::transport`
    /// at the top of every `process()` call so canon lane scheduling
    /// follows the DAW's master clock.
    transport: Arc<Transport>,
    /// Companion orchestrator owning the canon + counterpoint lanes.
    /// Shared with the editor for IPC config changes (`canon_configure`
    /// etc.); editor handlers use `try_lock` to avoid blocking the
    /// audio thread's per-block tick.
    companion: Arc<Mutex<Companion>>,
    guitar_input: Option<GuitarInput>,
    sample_rate: f32,

    /// Shared with the editor for noteUpdate emission. Audio thread
    /// writes on every send_harmonized_note_{on,off}; editor frame
    /// loop reads on each tick.
    note_state: Arc<Mutex<PluginNoteState>>,

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
        // Build engine, transport, and Companion in the same order
        // Tauri's AppState::default() uses (state.rs:152-168). The
        // WorldState wraps Arc clones of both so the Companion sees
        // the same engine instance the audio thread mutates.
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::DiatonicThirds,
        )));
        let transport = Transport::new(48_000);
        let world = WorldState::new(Arc::clone(&transport), Arc::clone(&engine));
        let companion = Arc::new(Mutex::new(Companion::new(world)));
        {
            let mut c = companion
                .lock()
                .expect("companion mutex poisoned at plugin init");
            c.lanes.push(Box::new(CanonLane::new()));
            c.lanes.push(Box::new(CounterpointLane::new()));
        }
        Self {
            params: Arc::new(ContrapunkParams::default()),
            engine,
            transport,
            companion,
            guitar_input: None,
            sample_rate: 48000.0,
            note_state: Arc::new(Mutex::new(PluginNoteState::default())),
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
    /// Only updates when values actually change. Holds the engine
    /// lock for the duration of the param sync — DAW params change
    /// at user-rate (<<1Hz typical), so the brief lock is fine.
    fn sync_params(&mut self) {
        let key = self.params.key.value();
        let mode = self.params.harmony_mode.value();
        let octave = self.params.octave_mode.value();
        let voices = self.params.voice_count.value();
        let vp = self.params.voice_position.value();
        let auto_key = self.params.auto_key.value();
        let vl = self.params.voice_leading.value();

        // Skip the lock if nothing changed — common path on most blocks.
        if key == self.last_key
            && mode == self.last_mode
            && octave == self.last_octave
            && voices == self.last_voices
            && vp == self.last_voice_pos
            && auto_key == self.last_auto_key
            && vl == self.last_voice_leading
        {
            return;
        }

        let mut engine = self.engine.lock().unwrap_or_else(|e| e.into_inner());
        if key != self.last_key {
            engine.set_key(key.to_contrapunk());
            self.last_key = key;
        }
        if mode != self.last_mode {
            engine.set_mode(mode.to_contrapunk());
            self.last_mode = mode;
        }
        if octave != self.last_octave {
            engine.set_octave_mode(octave.to_contrapunk());
            self.last_octave = octave;
        }
        if voices != self.last_voices {
            engine.set_voice_count(voices as usize);
            self.last_voices = voices;
        }
        if vp != self.last_voice_pos {
            engine.set_voice_position(vp as usize);
            self.last_voice_pos = vp;
        }
        if auto_key != self.last_auto_key {
            engine.set_auto_key(auto_key);
            self.last_auto_key = auto_key;
        }
        if vl != self.last_voice_leading {
            engine.set_voice_leading_enabled(vl);
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
        let input_midi = u8::from(note);
        let harmonized = {
            let mut engine = self.engine.lock().unwrap_or_else(|e| e.into_inner());
            engine.harmonize_note_on(note)
        };
        // Track for the editor's noteUpdate emission. Audio-thread
        // mutex; the lock is held briefly and the editor's read on
        // on_frame is best-effort. Lock failures degrade gracefully
        // (UI just stays on its last frame).
        if let Ok(mut s) = self.note_state.lock() {
            s.input_notes.insert(input_midi);
            for (i, &h_note) in harmonized.iter().enumerate() {
                // Skip i=0 — the engine returns the input pitch as the
                // first element; that belongs in input_notes only.
                if i == 0 {
                    continue;
                }
                s.harmony_notes.insert(u8::from(h_note));
            }
        }
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

    /// Mirror nih-plug's host-provided `Transport` snapshot into our
    /// sample-driven `contrapunk_transport::Transport`. Reads bpm /
    /// play-state / pos_samples once per `process()` call. Loop and
    /// locate events surface as a `pos_samples` jump bigger than a
    /// few buffers — re-seat sample_pos via `set_sample_pos` so
    /// beat-crossings stay aligned. Lock-free atomic writes only.
    fn sync_dawtransport(&self, context: &mut impl ProcessContext<Self>) {
        let dt = context.transport();
        if let Some(tempo) = dt.tempo {
            self.transport.set_bpm(tempo);
        }
        if let (Some(num), Some(_den)) = (dt.time_sig_numerator, dt.time_sig_denominator) {
            // Contrapunk's Transport stores numerator + beat-unit
            // separately; nih-plug uses i32 for both, we clamp.
            self.transport
                .set_time_signature(num.max(1) as u8, _den.max(1) as u8);
        }
        match (dt.playing, self.transport.is_running()) {
            (true, false) => self.transport.play(),
            (false, true) => self.transport.stop(),
            _ => {}
        }
        if let Some(host_pos) = dt.pos_samples() {
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

    /// Dispatch one MIDI input event through the Companion's lanes.
    /// Returns `suppress_default` from `on_input_tagged` so the caller
    /// can skip the regular harmony path when a lane fully owns this
    /// event. Lock contention with the editor falls through to a
    /// no-op (rare: editor IPC is user-rate, ~10Hz max).
    fn companion_on_input(
        &mut self,
        ev: contrapunk_companion::InputEvent,
        timing: u32,
        context: &mut impl ProcessContext<Self>,
    ) -> bool {
        let (tagged, suppress) = {
            let Ok(mut c) = self.companion.try_lock() else {
                return false; // editor busy; skip companion this event
            };
            c.on_input_tagged(ev, &self.engine)
        };
        self.dispatch_tagged_ops(&tagged, timing, context);
        suppress
    }

    /// Drain one block's worth of companion `tick_tagged` emissions.
    /// Called once per `process()` after all MIDI input events have
    /// been routed.
    fn companion_tick(&mut self, context: &mut impl ProcessContext<Self>) {
        let tagged = {
            let Ok(mut c) = self.companion.try_lock() else {
                return;
            };
            c.tick_tagged(&self.engine)
        };
        if !tagged.is_empty() {
            self.dispatch_tagged_ops(&tagged, 0, context);
        }
    }

    /// Translate a slice of `(lane_tag, DispatchOp)` into nih-plug
    /// `NoteEvent`s and emit them via `context.send_event`. Also
    /// updates the per-lane note-state HashSets so the editor's
    /// `noteUpdate` payload colors the Piano correctly.
    ///
    /// MPE channel mapping: ch 2 = canon lane, ch 3 = counterpoint
    /// lane, ch 1 reserved for the player melody. Future lanes get
    /// added to the match.
    fn dispatch_tagged_ops(
        &mut self,
        tagged: &[(&'static str, contrapunk_companion::DispatchOp)],
        timing: u32,
        context: &mut impl ProcessContext<Self>,
    ) {
        use contrapunk_companion::DispatchOp;
        for (lane, op) in tagged {
            match op {
                DispatchOp::NoteOn {
                    note,
                    velocity,
                    channel,
                    ..
                } => {
                    let mpe_ch = match *lane {
                        "canon" => 2,
                        "counterpoint" => 3,
                        _ => (*channel + 1).min(15),
                    };
                    context.send_event(NoteEvent::NoteOn {
                        timing,
                        voice_id: None,
                        channel: mpe_ch,
                        note: *note,
                        velocity: *velocity as f32 / 127.0,
                    });
                    if let Ok(mut s) = self.note_state.lock() {
                        match *lane {
                            "canon" => {
                                s.canon_notes.insert(*note);
                            }
                            "counterpoint" => {
                                s.counterpoint_notes.insert(*note);
                            }
                            _ => {}
                        }
                        s.harmony_notes.insert(*note);
                    }
                }
                DispatchOp::NoteOff { note, channel, .. } => {
                    let mpe_ch = match *lane {
                        "canon" => 2,
                        "counterpoint" => 3,
                        _ => (*channel + 1).min(15),
                    };
                    context.send_event(NoteEvent::NoteOff {
                        timing,
                        voice_id: None,
                        channel: mpe_ch,
                        note: *note,
                        velocity: 0.0,
                    });
                    if let Ok(mut s) = self.note_state.lock() {
                        match *lane {
                            "canon" => {
                                s.canon_notes.remove(note);
                            }
                            "counterpoint" => {
                                s.counterpoint_notes.remove(note);
                            }
                            _ => {}
                        }
                        s.harmony_notes.remove(note);
                    }
                }
                DispatchOp::AllNotesOff { .. } => {
                    // Broadcast on every MPE channel — the DAW will
                    // route to whichever synth is downstream.
                    for ch in 0u8..16 {
                        context.send_event(NoteEvent::MidiCC {
                            timing,
                            channel: ch,
                            cc: 123, // All Notes Off
                            value: 0.0,
                        });
                    }
                    if let Ok(mut s) = self.note_state.lock() {
                        s.canon_notes.clear();
                        s.counterpoint_notes.clear();
                    }
                }
            }
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
        let input_midi = u8::from(note);
        let released = {
            let mut engine = self.engine.lock().unwrap_or_else(|e| e.into_inner());
            engine.harmonize_note_off(note)
        };
        if let Ok(mut s) = self.note_state.lock() {
            s.input_notes.remove(&input_midi);
            for (i, &h_note) in released.iter().enumerate() {
                if i == 0 {
                    continue;
                }
                s.harmony_notes.remove(&u8::from(h_note));
            }
        }
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(editor::create_editor(
            self.params.clone(),
            Arc::clone(&self.note_state),
            Arc::clone(&self.companion),
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

        // Sync DAW transport → our sample-driven Transport so canon
        // lane scheduling follows the host clock. Per-block snapshot:
        // bpm, play/stop, sample_pos. Loop/jump discontinuities
        // detected via delta-bigger-than-buffer-size heuristic.
        self.sync_dawtransport(context);

        // Advance our clock by this block. `sync_dawtransport` may
        // have already seeked sample_pos to match the DAW; advance
        // applies on top so beat-crossings fire normally for this
        // block's worth of samples.
        let frames = buffer.samples() as u32;
        self.transport.advance(frames);

        let input_mode = self.params.input_mode.value();

        match input_mode {
            PluginInputMode::Midi => {
                // MIDI-to-MIDI: each incoming MIDI event runs through
                // the Companion (canon + counterpoint lanes) AND, if
                // the companion doesn't suppress, the regular harmony
                // path. Mirrors `commands/engine.rs:600-633` in Tauri.
                while let Some(event) = context.next_event() {
                    match event {
                        NoteEvent::NoteOn {
                            timing,
                            note,
                            velocity,
                            ..
                        } => {
                            let suppress = self.companion_on_input(
                                contrapunk_companion::InputEvent::NoteOn {
                                    note,
                                    velocity: (velocity * 127.0) as u8,
                                    channel: 0,
                                },
                                timing,
                                context,
                            );
                            if !suppress {
                                if let Ok(wmidi_note) = wmidi::Note::try_from(note) {
                                    self.send_harmonized_note_on(
                                        timing, wmidi_note, velocity, context,
                                    );
                                }
                            }
                        }
                        NoteEvent::NoteOff {
                            timing,
                            note,
                            velocity,
                            ..
                        } => {
                            let suppress = self.companion_on_input(
                                contrapunk_companion::InputEvent::NoteOff { note, channel: 0 },
                                timing,
                                context,
                            );
                            if !suppress {
                                if let Ok(wmidi_note) = wmidi::Note::try_from(note) {
                                    self.send_harmonized_note_off(
                                        timing, wmidi_note, velocity, context,
                                    );
                                }
                            }
                        }
                        // Forward other events unchanged
                        other => context.send_event(other),
                    }
                }

                // Drain scheduled / delayed canon emissions for this
                // block. `tick_tagged` returns the ops the lanes want
                // dispatched this tick — translate each to a NoteEvent
                // at timing=0 (block start). Sample-accurate scheduling
                // within the block is a v1.4 problem.
                self.companion_tick(context);
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
