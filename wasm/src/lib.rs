//! WASM Bridge for Contrapunk Harmony Engine
//!
//! Exposes the core harmony engine to JavaScript/TypeScript via wasm-bindgen.
//! This crate wraps `contrapunk::harmony::HarmonyEngine` and provides
//! string-based APIs suitable for the Svelte UI adapter layer.

use wasm_bindgen::prelude::*;

use contrapunk::harmony::VoiceLeadingStyle;
use contrapunk::harmony::{
    CounterpointSpecies, CounterpointStrictness, HarmonyEngine, HarmonyMode, Key, OctaveMode,
    ScaleMode,
};
use contrapunk::humanize::{DelayQueue, HumanizeConfig, Humanizer, Metronome};
use contrapunk::preset::PresetManager;

/// Log to browser console from Rust WASM.
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()))
}

// Initialize panic hook for better error messages in the browser console.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// === Helper: string-to-enum conversions ===

fn parse_key(s: &str) -> Result<Key, JsValue> {
    match s {
        "C" => Ok(Key::C),
        "Db" | "C#" => Ok(Key::Db),
        "D" => Ok(Key::D),
        "Eb" | "D#" => Ok(Key::Eb),
        "E" => Ok(Key::E),
        "F" => Ok(Key::F),
        "Gb" | "F#" => Ok(Key::Gb),
        "G" => Ok(Key::G),
        "Ab" | "G#" => Ok(Key::Ab),
        "A" => Ok(Key::A),
        "Bb" | "A#" => Ok(Key::Bb),
        "B" => Ok(Key::B),
        _ => Err(JsValue::from_str(&format!("Unknown key: {}", s))),
    }
}

fn parse_mode(s: &str) -> Result<HarmonyMode, JsValue> {
    match s {
        "PassThrough" => Ok(HarmonyMode::PassThrough),
        "DiatonicThirds" => Ok(HarmonyMode::DiatonicThirds),
        "DiatonicFourths" => Ok(HarmonyMode::DiatonicFourths),
        "RandomBelow" => Ok(HarmonyMode::RandomBelow),
        "RandomBelowNoSeconds" => Ok(HarmonyMode::RandomBelowNoSeconds),
        "ContraryMotion" => Ok(HarmonyMode::ContraryMotion),
        "StrictCounterpoint" => Ok(HarmonyMode::StrictCounterpoint),
        "BarryHarris" => Ok(HarmonyMode::BarryHarris),
        "FunctionalHarmony" => Ok(HarmonyMode::FunctionalHarmony),
        "BachChorale" => Ok(HarmonyMode::BachChorale),
        _ => Err(JsValue::from_str(&format!("Unknown mode: {}", s))),
    }
}

fn parse_scale_mode(s: &str) -> Result<ScaleMode, JsValue> {
    match s {
        // Diatonic
        "Ionian" => Ok(ScaleMode::Ionian),
        "Dorian" => Ok(ScaleMode::Dorian),
        "Phrygian" => Ok(ScaleMode::Phrygian),
        "Lydian" => Ok(ScaleMode::Lydian),
        "Mixolydian" => Ok(ScaleMode::Mixolydian),
        "Aeolian" => Ok(ScaleMode::Aeolian),
        "Locrian" => Ok(ScaleMode::Locrian),
        // Harmonic Minor
        "HarmonicMinor" => Ok(ScaleMode::HarmonicMinor),
        "LocrianNat6" => Ok(ScaleMode::LocrianNat6),
        "IonianAug" => Ok(ScaleMode::IonianAug),
        "DorianSharp4" => Ok(ScaleMode::DorianSharp4),
        "PhrygianDominant" => Ok(ScaleMode::PhrygianDominant),
        "LydianSharp2" => Ok(ScaleMode::LydianSharp2),
        "SuperLocrianDim" => Ok(ScaleMode::SuperLocrianDim),
        // Melodic Minor
        "MelodicMinor" => Ok(ScaleMode::MelodicMinor),
        "DorianFlat2" => Ok(ScaleMode::DorianFlat2),
        "LydianAug" => Ok(ScaleMode::LydianAug),
        "LydianDominant" => Ok(ScaleMode::LydianDominant),
        "MixolydianFlat6" => Ok(ScaleMode::MixolydianFlat6),
        "LocrianNat2" => Ok(ScaleMode::LocrianNat2),
        "SuperLocrian" => Ok(ScaleMode::SuperLocrian),
        // Harmonic Major
        "HarmonicMajor" => Ok(ScaleMode::HarmonicMajor),
        "DorianFlat5" => Ok(ScaleMode::DorianFlat5),
        "PhrygianFlat4" => Ok(ScaleMode::PhrygianFlat4),
        "LydianFlat3" => Ok(ScaleMode::LydianFlat3),
        "MixolydianFlat2" => Ok(ScaleMode::MixolydianFlat2),
        "LydianAugSharp2" => Ok(ScaleMode::LydianAugSharp2),
        "LocrianDoubleFlat7" => Ok(ScaleMode::LocrianDoubleFlat7),
        // Double Harmonic
        "DoubleHarmonic" => Ok(ScaleMode::DoubleHarmonic),
        "LydianSharp2Sharp6" => Ok(ScaleMode::LydianSharp2Sharp6),
        "Ultraphrygian" => Ok(ScaleMode::Ultraphrygian),
        "HungarianMinor" => Ok(ScaleMode::HungarianMinor),
        "Oriental" => Ok(ScaleMode::Oriental),
        "IonianSharp2Sharp5" => Ok(ScaleMode::IonianSharp2Sharp5),
        "LocrianDoubleFlat3DoubleFlat7" => Ok(ScaleMode::LocrianDoubleFlat3DoubleFlat7),
        // Pentatonic
        "MajorPentatonic" => Ok(ScaleMode::MajorPentatonic),
        "MinorPentatonic" => Ok(ScaleMode::MinorPentatonic),
        "Hirajoshi" => Ok(ScaleMode::Hirajoshi),
        "InSen" => Ok(ScaleMode::InSen),
        "Iwato" => Ok(ScaleMode::Iwato),
        "Yo" => Ok(ScaleMode::Yo),
        "Kumoi" => Ok(ScaleMode::Kumoi),
        "Pelog" => Ok(ScaleMode::Pelog),
        // Blues & Bebop
        "MinorBlues" => Ok(ScaleMode::MinorBlues),
        "MajorBlues" => Ok(ScaleMode::MajorBlues),
        "BebopDominant" => Ok(ScaleMode::BebopDominant),
        // Symmetric
        "WholeTone" => Ok(ScaleMode::WholeTone),
        "DiminishedWholeHalf" => Ok(ScaleMode::DiminishedWholeHalf),
        "DiminishedHalfWhole" => Ok(ScaleMode::DiminishedHalfWhole),
        "AugmentedHex" => Ok(ScaleMode::AugmentedHex),
        // World
        "Enigmatic" => Ok(ScaleMode::Enigmatic),
        "NeapolitanMinor" => Ok(ScaleMode::NeapolitanMinor),
        "NeapolitanMajor" => Ok(ScaleMode::NeapolitanMajor),
        "Persian" => Ok(ScaleMode::Persian),
        "HungarianMajor" => Ok(ScaleMode::HungarianMajor),
        // Barry Harris
        "BHMajor6thDim" => Ok(ScaleMode::BHMajor6thDim),
        "BHMinor6thDim" => Ok(ScaleMode::BHMinor6thDim),
        _ => Err(JsValue::from_str(&format!("Unknown scale mode: {}", s))),
    }
}

fn parse_octave_mode(s: &str) -> Result<OctaveMode, JsValue> {
    match s {
        "None" => Ok(OctaveMode::None),
        "Spread" => Ok(OctaveMode::Spread),
        "BassTrebleSplit" => Ok(OctaveMode::BassTrebleSplit),
        "Mirror" => Ok(OctaveMode::Mirror),
        _ => Err(JsValue::from_str(&format!("Unknown octave mode: {}", s))),
    }
}

fn parse_voice_leading_style(s: &str) -> Result<VoiceLeadingStyle, JsValue> {
    match s {
        "Free" => Ok(VoiceLeadingStyle::Free),
        "Palestrina" => Ok(VoiceLeadingStyle::Palestrina),
        "BachChorale" => Ok(VoiceLeadingStyle::BachChorale),
        "Jazz" => Ok(VoiceLeadingStyle::Jazz),
        _ => Err(JsValue::from_str(&format!(
            "Unknown voice leading style: {}",
            s
        ))),
    }
}

fn parse_counterpoint_species(s: &str) -> Result<CounterpointSpecies, JsValue> {
    match s {
        "Species1" | "species1" | "1" => Ok(CounterpointSpecies::Species1),
        "Species2" | "species2" | "2" => Ok(CounterpointSpecies::Species2),
        "Species3" | "species3" | "3" => Ok(CounterpointSpecies::Species3),
        "Species4" | "species4" | "4" => Ok(CounterpointSpecies::Species4),
        _ => Err(JsValue::from_str(&format!(
            "Unknown counterpoint species: {}",
            s
        ))),
    }
}

fn parse_counterpoint_strictness(s: &str) -> Result<CounterpointStrictness, JsValue> {
    match s {
        "Relaxed" | "relaxed" => Ok(CounterpointStrictness::Relaxed),
        "Strict" | "strict" => Ok(CounterpointStrictness::Strict),
        _ => Err(JsValue::from_str(&format!(
            "Unknown counterpoint strictness: {}",
            s
        ))),
    }
}

fn counterpoint_species_to_string(s: CounterpointSpecies) -> &'static str {
    match s {
        CounterpointSpecies::Species1 => "Species1",
        CounterpointSpecies::Species2 => "Species2",
        CounterpointSpecies::Species3 => "Species3",
        CounterpointSpecies::Species4 => "Species4",
    }
}

fn counterpoint_strictness_to_string(s: CounterpointStrictness) -> &'static str {
    match s {
        CounterpointStrictness::Relaxed => "Relaxed",
        CounterpointStrictness::Strict => "Strict",
    }
}

// === Enum-to-string helpers ===

fn key_to_string(key: Key) -> &'static str {
    match key {
        Key::C => "C",
        Key::Db => "Db",
        Key::D => "D",
        Key::Eb => "Eb",
        Key::E => "E",
        Key::F => "F",
        Key::Gb => "Gb",
        Key::G => "G",
        Key::Ab => "Ab",
        Key::A => "A",
        Key::Bb => "Bb",
        Key::B => "B",
    }
}

fn mode_to_string(mode: HarmonyMode) -> &'static str {
    match mode {
        HarmonyMode::PassThrough => "PassThrough",
        HarmonyMode::DiatonicThirds => "DiatonicThirds",
        HarmonyMode::DiatonicFourths => "DiatonicFourths",
        HarmonyMode::RandomBelow => "RandomBelow",
        HarmonyMode::RandomBelowNoSeconds => "RandomBelowNoSeconds",
        HarmonyMode::ContraryMotion => "ContraryMotion",
        HarmonyMode::StrictCounterpoint => "StrictCounterpoint",
        HarmonyMode::BarryHarris => "BarryHarris",
        HarmonyMode::FunctionalHarmony => "FunctionalHarmony",
        HarmonyMode::BachChorale => "BachChorale",
    }
}

fn scale_mode_to_string(mode: ScaleMode) -> &'static str {
    match mode {
        // Diatonic
        ScaleMode::Ionian => "Ionian",
        ScaleMode::Dorian => "Dorian",
        ScaleMode::Phrygian => "Phrygian",
        ScaleMode::Lydian => "Lydian",
        ScaleMode::Mixolydian => "Mixolydian",
        ScaleMode::Aeolian => "Aeolian",
        ScaleMode::Locrian => "Locrian",
        // Harmonic Minor
        ScaleMode::HarmonicMinor => "HarmonicMinor",
        ScaleMode::LocrianNat6 => "LocrianNat6",
        ScaleMode::IonianAug => "IonianAug",
        ScaleMode::DorianSharp4 => "DorianSharp4",
        ScaleMode::PhrygianDominant => "PhrygianDominant",
        ScaleMode::LydianSharp2 => "LydianSharp2",
        ScaleMode::SuperLocrianDim => "SuperLocrianDim",
        // Melodic Minor
        ScaleMode::MelodicMinor => "MelodicMinor",
        ScaleMode::DorianFlat2 => "DorianFlat2",
        ScaleMode::LydianAug => "LydianAug",
        ScaleMode::LydianDominant => "LydianDominant",
        ScaleMode::MixolydianFlat6 => "MixolydianFlat6",
        ScaleMode::LocrianNat2 => "LocrianNat2",
        ScaleMode::SuperLocrian => "SuperLocrian",
        // Harmonic Major
        ScaleMode::HarmonicMajor => "HarmonicMajor",
        ScaleMode::DorianFlat5 => "DorianFlat5",
        ScaleMode::PhrygianFlat4 => "PhrygianFlat4",
        ScaleMode::LydianFlat3 => "LydianFlat3",
        ScaleMode::MixolydianFlat2 => "MixolydianFlat2",
        ScaleMode::LydianAugSharp2 => "LydianAugSharp2",
        ScaleMode::LocrianDoubleFlat7 => "LocrianDoubleFlat7",
        // Double Harmonic
        ScaleMode::DoubleHarmonic => "DoubleHarmonic",
        ScaleMode::LydianSharp2Sharp6 => "LydianSharp2Sharp6",
        ScaleMode::Ultraphrygian => "Ultraphrygian",
        ScaleMode::HungarianMinor => "HungarianMinor",
        ScaleMode::Oriental => "Oriental",
        ScaleMode::IonianSharp2Sharp5 => "IonianSharp2Sharp5",
        ScaleMode::LocrianDoubleFlat3DoubleFlat7 => "LocrianDoubleFlat3DoubleFlat7",
        // Pentatonic
        ScaleMode::MajorPentatonic => "MajorPentatonic",
        ScaleMode::MinorPentatonic => "MinorPentatonic",
        ScaleMode::Hirajoshi => "Hirajoshi",
        ScaleMode::InSen => "InSen",
        ScaleMode::Iwato => "Iwato",
        ScaleMode::Yo => "Yo",
        ScaleMode::Kumoi => "Kumoi",
        ScaleMode::Pelog => "Pelog",
        // Blues & Bebop
        ScaleMode::MinorBlues => "MinorBlues",
        ScaleMode::MajorBlues => "MajorBlues",
        ScaleMode::BebopDominant => "BebopDominant",
        // Symmetric
        ScaleMode::WholeTone => "WholeTone",
        ScaleMode::DiminishedWholeHalf => "DiminishedWholeHalf",
        ScaleMode::DiminishedHalfWhole => "DiminishedHalfWhole",
        ScaleMode::AugmentedHex => "AugmentedHex",
        // World
        ScaleMode::Enigmatic => "Enigmatic",
        ScaleMode::NeapolitanMinor => "NeapolitanMinor",
        ScaleMode::NeapolitanMajor => "NeapolitanMajor",
        ScaleMode::Persian => "Persian",
        ScaleMode::HungarianMajor => "HungarianMajor",
        // Barry Harris
        ScaleMode::BHMajor6thDim => "BHMajor6thDim",
        ScaleMode::BHMinor6thDim => "BHMinor6thDim",
    }
}

fn octave_mode_to_string(mode: OctaveMode) -> &'static str {
    match mode {
        OctaveMode::None => "None",
        OctaveMode::Spread => "Spread",
        OctaveMode::BassTrebleSplit => "BassTrebleSplit",
        OctaveMode::Mirror => "Mirror",
    }
}

fn voice_leading_style_to_string(style: VoiceLeadingStyle) -> &'static str {
    match style {
        VoiceLeadingStyle::Free => "Free",
        VoiceLeadingStyle::Palestrina => "Palestrina",
        VoiceLeadingStyle::BachChorale => "BachChorale",
        VoiceLeadingStyle::Jazz => "Jazz",
    }
}

// === State type for serialization ===

#[derive(serde::Serialize)]
struct SuggestionScoreJs {
    note: u8,
    score: f32,
}

#[derive(serde::Serialize)]
struct EngineStateJs {
    key: &'static str,
    mode: &'static str,
    mode_number: u8,
    scale_mode: &'static str,
    octave_mode: &'static str,
    voice_leading_enabled: bool,
    voice_leading_style: &'static str,
    interchange_enabled: bool,
    borrowing_range: u8,
    voice_position: usize,
    voice_count: usize,
    counterpoint_species: &'static str,
    counterpoint_strictness: &'static str,
}

#[derive(serde::Serialize)]
struct NoteStateJs {
    input_notes: Vec<u8>,
    harmony_notes: Vec<u8>,
    borrowed_notes: Vec<u8>,
    chord_name: String,
    last_borrowed_from: String,
}

#[derive(serde::Serialize)]
struct PresetJs {
    name: String,
    persona: String,
    genre: String,
    is_builtin: bool,
}

// === WASM-exported Engine wrapper ===

/// Structured result of a single `Engine::tick()` call, serialized to JS.
///
/// The UI's requestAnimationFrame loop drives `tick()` once per frame.
/// Each tick may advance the beat clock across a beat boundary (metronome
/// click), release queued humanized notes whose delay has elapsed, and/or
/// report the current beat-phase position for the UI beat indicator.
#[derive(serde::Serialize, Default)]
struct TickResultJs {
    /// Current beat-phase position within the bar (0..beats_per_bar).
    beat_position: f64,
    /// Current beat number (0-indexed) the clock is sitting on.
    beat_number: u8,
    /// If a beat boundary was crossed this frame, the integer beat index
    /// that was just entered (0-indexed). Used to drive downbeat accent.
    beat_crossed: Option<u8>,
    /// Metronome Note-On bytes ready to emit this frame (MIDI Ch10).
    metronome_on: Option<Vec<u8>>,
    /// Metronome Note-Off bytes that became due this frame.
    metronome_off: Option<Vec<u8>>,
    /// Humanized harmony notes (other than the immediate pass-through
    /// notes already returned by `note_on`/`note_off`) whose delay has
    /// elapsed this frame. Each `[status, note, velocity]` triplet is a
    /// complete MIDI message ready to send to an output port.
    scheduled_notes: Vec<ScheduledMidiJs>,
    /// Whether humanization is currently enabled (master flag).
    humanize_enabled: bool,
    /// Whether the clock is currently running.
    running: bool,
    /// BPM of the beat clock.
    bpm: f64,
}

/// A single humanized MIDI event ready to send this frame.
#[derive(serde::Serialize)]
struct ScheduledMidiJs {
    /// Output port index the caller should route this message to.
    port: usize,
    /// Full MIDI message bytes (3-byte Note-On/Note-Off on the humanized
    /// channel and with the humanized velocity).
    bytes: Vec<u8>,
}

#[wasm_bindgen]
pub struct Engine {
    inner: HarmonyEngine,
    presets: PresetManager,
    /// Track notes that were played through note_on for note state reporting
    last_input_notes: Vec<u8>,
    last_harmony_notes: Vec<u8>,
    /// Configuration for the next-note suggestion scorer
    suggestion_config: contrapunk::harmony::suggestion::SuggestionConfig,
    /// Beat-aware humanizer wrapping BeatClock + HumanizeConfig.
    humanizer: Humanizer,
    /// Delay queue for humanized Note-On/Note-Off events (swing/jitter).
    delay_queue: DelayQueue,
    /// Metronome click generator (MIDI channel 10 woodblock).
    metronome: Metronome,
    /// First `now_ms` the WASM Engine saw. All timestamps are normalized
    /// relative to this so beat math stays stable even when JS passes
    /// absolute `performance.now()` values (which can be large f64s).
    epoch_ms: Option<f64>,
    /// Pending metronome Note-Off bytes scheduled for a future tick.
    /// Each entry is `(due_ms, midi_bytes)` in engine-relative time.
    pending_metronome_offs: Vec<(f64, Vec<u8>)>,
    /// Last engine-relative `now_ms` observed in `tick()`. Used by
    /// `humanized_note_on/off` as the reference timestamp when pushing
    /// onto the delay queue; RAF ticks happen ~60Hz so this is always
    /// within ~16ms of real time.
    last_tick_ms: f64,
}

#[wasm_bindgen]
impl Engine {
    /// Create a new Engine with default settings (C major, PassThrough).
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: HarmonyEngine::with_voices(Key::C, HarmonyMode::PassThrough, 4),
            presets: PresetManager::new(),
            last_input_notes: Vec::new(),
            last_harmony_notes: Vec::new(),
            suggestion_config: contrapunk::harmony::suggestion::SuggestionConfig::default(),
            humanizer: Humanizer::new(HumanizeConfig::default()),
            delay_queue: DelayQueue::new(),
            metronome: Metronome::new(),
            epoch_ms: None,
            pending_metronome_offs: Vec::new(),
            last_tick_ms: 0.0,
        }
    }

    /// Clear tracked note state (call after config changes that invalidate active harmonies).
    pub fn clear_notes(&mut self) {
        self.last_input_notes.clear();
        self.last_harmony_notes.clear();
    }

    /// Set the musical key (e.g. "C", "Db", "F#").
    pub fn set_key(&mut self, key: &str) -> Result<(), JsValue> {
        let k = parse_key(key)?;
        self.inner.set_key(k);
        self.clear_notes();
        Ok(())
    }

    /// Set the harmony mode (e.g. "PassThrough", "DiatonicThirds").
    pub fn set_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        let m = parse_mode(mode)?;
        self.inner.set_mode(m);
        self.clear_notes();
        Ok(())
    }

    /// Set the scale mode (e.g. "Ionian", "Dorian", "HarmonicMinor").
    pub fn set_scale_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        let sm = parse_scale_mode(mode)?;
        self.inner.set_scale_mode(sm);
        self.clear_notes();
        Ok(())
    }

    /// Set the octave mode (e.g. "None", "Spread", "Mirror").
    pub fn set_octave_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        let om = parse_octave_mode(mode)?;
        self.inner.set_octave_mode(om);
        self.clear_notes();
        Ok(())
    }

    /// Configure voice leading (enabled flag + style string).
    pub fn set_voice_leading(&mut self, enabled: bool, style: &str) -> Result<(), JsValue> {
        let vl_style = parse_voice_leading_style(style)?;
        self.inner.set_voice_leading_enabled(enabled);
        self.inner.set_voice_leading_style(vl_style);
        self.clear_notes();
        Ok(())
    }

    /// Configure modal interchange (enabled flag + borrowing range 1-5).
    pub fn set_interchange(&mut self, enabled: bool, range: u8) -> Result<(), JsValue> {
        self.inner.set_interchange_enabled(enabled);
        self.inner.set_borrowing_range(range);
        self.clear_notes();
        Ok(())
    }

    /// Set the voice position (which output slot carries the melody).
    pub fn set_voice_position(&mut self, position: usize) -> Result<(), JsValue> {
        self.inner.set_voice_position(position);
        Ok(())
    }

    /// Set the number of output voices (1 = melody only, 2+ = melody + harmonies).
    pub fn set_voice_count(&mut self, count: usize) -> Result<(), JsValue> {
        self.inner.set_voice_count(count);
        Ok(())
    }

    /// Enable or disable auto-key detection.
    pub fn set_auto_key(&mut self, enabled: bool) -> Result<(), JsValue> {
        self.inner.set_auto_key(enabled);
        Ok(())
    }

    /// Set the counterpoint species (`"Species1"` through `"Species4"`).
    ///
    /// Only active when the harmony mode is `StrictCounterpoint`. Species 2-4
    /// require a beat-phase clock; since the WASM build has no internal
    /// metronome, these species currently behave like Species 1 unless the
    /// host explicitly calls `set_counterpoint_beat_phase` each frame.
    pub fn set_counterpoint_species(&mut self, species: &str) -> Result<(), JsValue> {
        let s = parse_counterpoint_species(species)?;
        self.inner.set_counterpoint_species(s);
        self.clear_notes();
        Ok(())
    }

    /// Set the counterpoint strictness (`"Relaxed"` or `"Strict"`).
    pub fn set_counterpoint_strictness(&mut self, strictness: &str) -> Result<(), JsValue> {
        let s = parse_counterpoint_strictness(strictness)?;
        self.inner.set_counterpoint_strictness(s);
        self.clear_notes();
        Ok(())
    }

    /// Set the counterpoint beat-phase position within the bar
    /// (`0.0 .. beats_per_bar`). Pass `None` (via JS `undefined`/`null` from
    /// the optional setter) to disable beat awareness and fall back to
    /// Species 1 behavior.
    pub fn set_counterpoint_beat_phase(&mut self, phase: Option<f64>) {
        self.inner.set_counterpoint_beat_phase(phase);
    }

    /// Returns the current key as a string (for UI to update after auto-detection).
    pub fn current_key(&self) -> String {
        format!("{}", self.inner.key())
    }

    /// Harmonize a single MIDI note number.
    /// Returns a JS array of MIDI note numbers (u8).
    pub fn harmonize(&mut self, note: u8) -> Result<Vec<u8>, JsValue> {
        let wmidi_note = wmidi::Note::from_u8_lossy(note);
        let results = self.inner.harmonize(wmidi_note);
        Ok(results.iter().map(|n| u8::from(*n)).collect())
    }

    /// Process a MIDI Note-On event.
    /// Returns a JS array of MIDI note numbers to sound.
    pub fn note_on(&mut self, note: u8) -> Result<Vec<u8>, JsValue> {
        let wmidi_note = wmidi::Note::from_u8_lossy(note);
        let results = self.inner.harmonize_note_on(wmidi_note);
        let result_u8: Vec<u8> = results.iter().map(|n| u8::from(*n)).collect();

        // Track for note state reporting
        if !self.last_input_notes.contains(&note) {
            self.last_input_notes.push(note);
        }
        // All notes beyond the first are harmony
        for &n in &result_u8[1..] {
            if !self.last_harmony_notes.contains(&n) {
                self.last_harmony_notes.push(n);
            }
        }

        Ok(result_u8)
    }

    /// Process a MIDI Note-Off event.
    /// Returns a JS array of MIDI note numbers to release.
    pub fn note_off(&mut self, note: u8) -> Result<Vec<u8>, JsValue> {
        let wmidi_note = wmidi::Note::from_u8_lossy(note);
        let results = self.inner.harmonize_note_off(wmidi_note);
        let result_u8: Vec<u8> = results.iter().map(|n| u8::from(*n)).collect();

        // Remove from tracked state
        self.last_input_notes.retain(|&n| n != note);
        for &n in &result_u8 {
            self.last_harmony_notes.retain(|&h| h != n);
        }

        Ok(result_u8)
    }

    /// Process a MIDI Note-On with humanization applied to harmony voices.
    ///
    /// The melody note (voice 0) is always returned with zero delay so the
    /// player hears their input in realtime. Harmony notes (voice 1+) pass
    /// through the humanizer: they get random velocity/jitter/swing, and
    /// any note with a non-zero delay is pushed into the internal delay
    /// queue to be released on a later `tick()`.
    ///
    /// Returns a JS object shaped like:
    /// ```ignore
    /// {
    ///   immediate: [{ port, bytes }, ...],   // send this frame
    ///   deferred_count: number,              // queued for later tick()s
    ///   input_note: u8,
    /// }
    /// ```
    pub fn humanized_note_on(&mut self, note: u8, velocity: u8) -> Result<JsValue, JsValue> {
        let wmidi_note = wmidi::Note::from_u8_lossy(note);
        let wmidi_vel = wmidi::Velocity::try_from(velocity.clamp(1, 127)).unwrap();

        let results = self.inner.harmonize_note_on(wmidi_note);
        let port_map: Vec<usize> = self.inner.last_port_map().to_vec();

        // Track for note state reporting.
        if !self.last_input_notes.contains(&note) {
            self.last_input_notes.push(note);
        }
        let all_u8: Vec<u8> = results.iter().map(|n| u8::from(*n)).collect();
        for &n in &all_u8[results.len().min(1)..] {
            if !self.last_harmony_notes.contains(&n) {
                self.last_harmony_notes.push(n);
            }
        }

        let mut immediate: Vec<ScheduledMidiJs> = Vec::new();
        // Engine-relative "now" — the most recent tick() timestamp.
        // RAF drives tick() ~60Hz so this is always within ~16ms of
        // real time. If `tick()` hasn't run yet, `last_tick_ms` is 0.0
        // and any scheduled notes will simply fire on the first tick.
        let t = self.last_tick_ms;

        let melody_port = port_map.first().copied().unwrap_or(0);
        for (i, &n) in results.iter().enumerate() {
            let port = port_map.get(i).copied().unwrap_or(i);
            let midi_note = u8::from(n);
            if i == 0 {
                // Melody — pass straight through with the input velocity.
                let _ = port;
                immediate.push(ScheduledMidiJs {
                    port: melody_port,
                    bytes: vec![0x90, midi_note, velocity],
                });
            } else {
                // Harmony — humanize. If humanization is disabled the
                // Humanizer returns delay_ms=0 and velocity unchanged.
                let hn = self
                    .humanizer
                    .humanize_note_on(n, wmidi::Channel::Ch1, wmidi_vel, port);
                if hn.delay_ms == 0 {
                    immediate.push(ScheduledMidiJs {
                        port: hn.port,
                        bytes: vec![
                            0x90 | (hn.channel.index() & 0x0f),
                            u8::from(hn.note),
                            u8::from(hn.velocity),
                        ],
                    });
                } else {
                    self.delay_queue.push(hn, t);
                }
            }
        }

        #[derive(serde::Serialize)]
        struct HumanizedNoteResult {
            immediate: Vec<ScheduledMidiJs>,
            deferred_count: usize,
            input_note: u8,
        }
        let deferred = results.len().saturating_sub(immediate.len());
        let result = HumanizedNoteResult {
            immediate,
            deferred_count: deferred,
            input_note: note,
        };
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Process a MIDI Note-Off with humanization applied to harmony voices.
    ///
    /// See [`humanized_note_on`] for the result shape. Note-Off timings
    /// mirror the Note-On humanization record (so a swung 8th note stays
    /// swung on release).
    pub fn humanized_note_off(&mut self, note: u8) -> Result<JsValue, JsValue> {
        let wmidi_note = wmidi::Note::from_u8_lossy(note);
        let results = self.inner.harmonize_note_off(wmidi_note);
        let port_map: Vec<usize> = self.inner.last_port_map().to_vec();

        let all_u8: Vec<u8> = results.iter().map(|n| u8::from(*n)).collect();
        self.last_input_notes.retain(|&n| n != note);
        for &n in &all_u8 {
            self.last_harmony_notes.retain(|&h| h != n);
        }

        let t = self.last_tick_ms;
        let wmidi_vel = wmidi::Velocity::try_from(64u8).unwrap();

        let mut immediate: Vec<ScheduledMidiJs> = Vec::new();
        for (i, &n) in results.iter().enumerate() {
            let port = port_map.get(i).copied().unwrap_or(i);
            let midi_note = u8::from(n);
            if i == 0 {
                // Melody — release immediately.
                immediate.push(ScheduledMidiJs {
                    port,
                    bytes: vec![0x80, midi_note, 0],
                });
            } else {
                let hn = self
                    .humanizer
                    .humanize_note_off(n, wmidi::Channel::Ch1, wmidi_vel, port);
                if hn.delay_ms == 0 {
                    immediate.push(ScheduledMidiJs {
                        port: hn.port,
                        bytes: vec![
                            0x80 | (hn.channel.index() & 0x0f),
                            u8::from(hn.note),
                            u8::from(hn.velocity),
                        ],
                    });
                } else {
                    self.delay_queue.push(hn, t);
                }
            }
        }

        #[derive(serde::Serialize)]
        struct HumanizedNoteResult {
            immediate: Vec<ScheduledMidiJs>,
            deferred_count: usize,
            input_note: u8,
        }
        let deferred = results.len().saturating_sub(immediate.len());
        let result = HumanizedNoteResult {
            immediate,
            deferred_count: deferred,
            input_note: note,
        };
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    // === Humanizer / Beat Clock / Metronome ===
    //
    // The WASM Engine owns its own `Humanizer` (native/Tauri each own
    // theirs separately). JS drives `tick()` via requestAnimationFrame,
    // passing `performance.now()` as a monotonic millisecond clock. On
    // every tick we:
    //   1. Normalize `now_ms` against the engine's epoch (first tick).
    //   2. Advance the BeatClock.
    //   3. Push the beat-phase position into the HarmonyEngine so that
    //      Counterpoint Species 2-4 can react to weak/strong beats.
    //   4. If the metronome is enabled and a beat boundary was crossed,
    //      emit a click and schedule a matching click-off.
    //   5. Drain any humanized harmony notes whose delay has elapsed.
    //
    // All delay-queue times are in engine-relative milliseconds to keep
    // arithmetic stable against large `performance.now()` values.

    /// Normalize a raw `performance.now()` stamp against the engine epoch.
    /// The first call seeds the epoch and starts the beat clock.
    fn normalize_now(&mut self, now_ms: f64) -> f64 {
        if self.epoch_ms.is_none() {
            self.epoch_ms = Some(now_ms);
            // Lazy-start the beat clock on the first tick so swing math is
            // anchored to when the UI actually began ticking, not when the
            // WASM module was constructed (which may be seconds earlier).
            self.humanizer.clock_mut().start(0.0);
        }
        now_ms - self.epoch_ms.unwrap_or(now_ms)
    }

    /// Drive the beat clock + metronome + humanize delay queue forward.
    ///
    /// Must be called on every animation frame with `performance.now()`.
    /// Returns a JSON-serializable object describing what happened this
    /// frame: beat position, any metronome click bytes, and any delayed
    /// humanized notes that are now due to be sent.
    ///
    /// The returned object shape (`TickResultJs`) is:
    /// ```ignore
    /// {
    ///   beat_position: f64,
    ///   beat_number: u8,
    ///   beat_crossed: u8 | null,
    ///   metronome_on: u8[] | null,
    ///   metronome_off: u8[] | null,
    ///   scheduled_notes: [{ port: number, bytes: u8[] }, ...],
    ///   humanize_enabled: bool,
    ///   running: bool,
    ///   bpm: f64,
    /// }
    /// ```
    pub fn tick(&mut self, now_ms: f64) -> Result<JsValue, JsValue> {
        let t = self.normalize_now(now_ms);
        self.last_tick_ms = t;

        // Advance beat clock.
        self.humanizer.tick(t);
        let beat_pos = self.humanizer.clock().beat_position();
        let beat_crossed = self.humanizer.clock().beat_crossed();

        // Feed beat-phase into Counterpoint Species 2-4.
        self.inner.set_counterpoint_beat_phase(Some(beat_pos));

        // Metronome click on beat crossing.
        let (mut metronome_on, mut metronome_off) = (None, None);
        if let Some(beat_num) = beat_crossed {
            if self.humanizer.config().metronome_enabled {
                let on_bytes = self.metronome.generate_click(beat_num);
                let off_bytes = self.metronome.generate_click_off(beat_num);
                metronome_on = Some(on_bytes);
                // Schedule the NoteOff ~50ms after the click so it sounds
                // percussive, not sustained.
                self.pending_metronome_offs.push((t + 50.0, off_bytes));
            }
        }

        // Drain any metronome NoteOffs whose time has arrived.
        let mut fired_offs: Vec<Vec<u8>> = Vec::new();
        self.pending_metronome_offs.retain(|(due, bytes)| {
            if *due <= t {
                fired_offs.push(bytes.clone());
                false
            } else {
                true
            }
        });
        if !fired_offs.is_empty() {
            // Collapse to a single NoteOff per tick (metronome is monophonic
            // per beat). If multiple piled up, the last one wins — that's
            // fine; it's still just silencing the woodblock.
            metronome_off = fired_offs.pop();
        }

        // Drain the humanized Note-On/Note-Off delay queue.
        let ready = self.delay_queue.drain_ready(t);
        let mut scheduled_notes: Vec<ScheduledMidiJs> = Vec::with_capacity(ready.len());
        for hn in ready {
            let status_byte: u8 = if hn.is_note_off { 0x80 } else { 0x90 };
            let msg = vec![
                status_byte | (hn.channel.index() & 0x0f),
                u8::from(hn.note),
                u8::from(hn.velocity),
            ];
            scheduled_notes.push(ScheduledMidiJs {
                port: hn.port,
                bytes: msg,
            });
        }

        let cfg = self.humanizer.config();
        let result = TickResultJs {
            beat_position: beat_pos,
            beat_number: beat_pos.floor() as u8,
            beat_crossed,
            metronome_on,
            metronome_off,
            scheduled_notes,
            humanize_enabled: cfg.enabled,
            running: self.humanizer.clock().running,
            bpm: cfg.bpm,
        };

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Master toggle for all humanization effects.
    ///
    /// When `false`, Note-On/Note-Off pass through unchanged even if
    /// swing/jitter/velocity-variation sub-toggles are on.
    pub fn set_humanize_enabled(&mut self, enabled: bool) {
        self.humanizer.config_mut().enabled = enabled;
    }

    /// Enable/disable timing jitter on harmony notes.
    pub fn set_jitter_enabled(&mut self, enabled: bool) {
        self.humanizer.config_mut().jitter_enabled = enabled;
    }

    /// Set the upper bound for per-note timing jitter (milliseconds).
    /// The lower bound tracks 1ms or `max.min(1)` whichever is smaller.
    pub fn set_timing_jitter(&mut self, max_ms: u16) {
        let cfg = self.humanizer.config_mut();
        cfg.jitter_max_ms = max_ms;
        if cfg.jitter_min_ms > max_ms {
            cfg.jitter_min_ms = max_ms;
        }
    }

    /// Enable/disable random velocity variation on harmony notes.
    pub fn set_velocity_enabled(&mut self, enabled: bool) {
        self.humanizer.config_mut().velocity_enabled = enabled;
    }

    /// Set the +/- range for random velocity jitter (0..127 scale).
    pub fn set_velocity_jitter(&mut self, variation: u8) {
        self.humanizer.config_mut().velocity_variation = variation;
    }

    /// Enable/disable duration extension on harmony Note-Offs.
    pub fn set_duration_enabled(&mut self, enabled: bool) {
        self.humanizer.config_mut().duration_enabled = enabled;
    }

    /// Set the max duration extension in ms (applied to Note-Off timing).
    pub fn set_duration_variation(&mut self, ms: u16) {
        self.humanizer.config_mut().duration_variation_ms = ms;
    }

    /// Enable/disable swing feel on off-beats.
    pub fn set_swing_enabled(&mut self, enabled: bool) {
        self.humanizer.config_mut().swing_enabled = enabled;
    }

    /// Set the swing amount. 0.0 = straight, 0.3 = light, 0.5 = jazz.
    pub fn set_swing(&mut self, amount: f32) {
        self.humanizer.config_mut().swing_amount = amount;
    }

    /// Set the tempo in BPM. Updates the beat clock without resetting
    /// beat position (so the metronome doesn't stutter on tempo changes).
    pub fn set_bpm(&mut self, bpm: f64) {
        let cfg = self.humanizer.config_mut();
        cfg.bpm = bpm;
        let beats_per_bar = cfg.beats_per_bar;
        let beat_unit = cfg.beat_unit;
        self.humanizer
            .clock_mut()
            .update_tempo(bpm, beats_per_bar, beat_unit);
    }

    /// Set the time signature (e.g. 4, 4 for 4/4 time).
    pub fn set_time_signature(&mut self, beats_per_bar: u8, beat_unit: u8) {
        let cfg = self.humanizer.config_mut();
        cfg.beats_per_bar = beats_per_bar;
        cfg.beat_unit = beat_unit;
        let bpm = cfg.bpm;
        self.humanizer
            .clock_mut()
            .update_tempo(bpm, beats_per_bar, beat_unit);
    }

    /// Enable/disable the metronome click track.
    pub fn set_metronome_enabled(&mut self, enabled: bool) {
        self.humanizer.config_mut().metronome_enabled = enabled;
        if !enabled {
            // Flush any pending note-offs so the last click isn't stuck on.
            self.pending_metronome_offs.clear();
        }
    }

    /// Current beat-phase position within the bar (for UI beat indicator).
    pub fn beat_position(&self) -> f64 {
        self.humanizer.clock().beat_position()
    }

    /// Current tempo in BPM.
    pub fn bpm(&self) -> f64 {
        self.humanizer.config().bpm
    }

    /// Whether the metronome click is currently enabled.
    pub fn is_metronome_enabled(&self) -> bool {
        self.humanizer.config().metronome_enabled
    }

    /// Whether humanization is currently enabled (master flag).
    pub fn is_humanize_enabled(&self) -> bool {
        self.humanizer.config().enabled
    }

    /// Full humanize configuration as a JS object (for UI round-tripping).
    pub fn get_humanize_config(&self) -> Result<JsValue, JsValue> {
        let cfg = self.humanizer.config();
        // Use `serde_wasm_bindgen` so the UI gets a plain JS object
        // shaped like Rust's `HumanizeConfig` (snake_case).
        serde_wasm_bindgen::to_value(cfg)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Bulk-update the humanize configuration from a JS object.
    ///
    /// Accepts the same snake_case shape as `get_humanize_config` returns.
    /// Fields omitted fall back to their current values.
    pub fn set_humanize_config(&mut self, config: JsValue) -> Result<(), JsValue> {
        let new_cfg: HumanizeConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        self.humanizer.update_config(new_cfg);
        Ok(())
    }

    /// Get the current engine state as a JS object.
    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        let state = EngineStateJs {
            key: key_to_string(self.inner.key()),
            mode: mode_to_string(self.inner.mode()),
            mode_number: self.inner.mode().number(),
            scale_mode: scale_mode_to_string(self.inner.scale_mode()),
            octave_mode: octave_mode_to_string(self.inner.octave_mode()),
            voice_leading_enabled: self.inner.voice_leading_enabled(),
            voice_leading_style: voice_leading_style_to_string(self.inner.voice_leading_style()),
            interchange_enabled: self.inner.interchange_enabled(),
            borrowing_range: self.inner.borrowing_range(),
            voice_position: self.inner.voice_position(),
            voice_count: self.inner.voice_count(),
            counterpoint_species: counterpoint_species_to_string(self.inner.counterpoint_species()),
            counterpoint_strictness: counterpoint_strictness_to_string(
                self.inner.counterpoint_strictness(),
            ),
        };

        serde_wasm_bindgen::to_value(&state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get the current note state as a JS object.
    pub fn get_note_state(&self) -> Result<JsValue, JsValue> {
        let borrowed_from = self
            .inner
            .last_borrowed_from()
            .map(|sm| scale_mode_to_string(sm).to_string())
            .unwrap_or_default();

        let state = NoteStateJs {
            input_notes: self.last_input_notes.clone(),
            harmony_notes: self.last_harmony_notes.clone(),
            borrowed_notes: Vec::new(), // Populated by routing layer
            chord_name: String::new(),  // Populated by chord detection
            last_borrowed_from: borrowed_from,
        };

        serde_wasm_bindgen::to_value(&state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// List all available presets (builtins + custom).
    pub fn list_presets(&self) -> Result<JsValue, JsValue> {
        let presets: Vec<PresetJs> = self
            .presets
            .all_presets()
            .iter()
            .map(|p| PresetJs {
                name: p.name.clone(),
                persona: p.persona.clone(),
                genre: p.genre.clone(),
                is_builtin: p.is_builtin,
            })
            .collect();

        serde_wasm_bindgen::to_value(&presets)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Load a preset by name, applying its settings to the engine.
    pub fn load_preset(&mut self, name: &str) -> Result<(), JsValue> {
        let all = self.presets.all_presets();
        let preset = all
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| JsValue::from_str(&format!("Preset not found: {}", name)))?;

        self.inner.set_key(preset.key);
        self.inner.set_mode(preset.harmony_mode);
        self.inner.set_scale_mode(preset.scale_mode);
        self.inner.set_octave_mode(preset.octave_mode);
        self.inner
            .set_voice_leading_enabled(preset.voice_leading_enabled);
        self.inner
            .set_voice_leading_style(preset.voice_leading_style);
        self.inner
            .set_interchange_enabled(preset.interchange_enabled);
        self.inner.set_borrowing_range(preset.borrowing_range);

        // Set active index
        if let Some(idx) = all.iter().position(|p| p.name == name) {
            self.presets.set_active(idx);
        }

        Ok(())
    }

    /// Save current engine settings as a custom preset.
    pub fn save_preset(&mut self, name: &str) -> Result<(), JsValue> {
        use contrapunk::humanize::HumanizeConfig;
        use contrapunk::preset::StylePreset;

        let preset = StylePreset {
            name: name.to_string(),
            persona: String::new(),
            genre: "Custom".to_string(),
            harmony_mode: self.inner.mode(),
            key: self.inner.key(),
            voice_leading_enabled: self.inner.voice_leading_enabled(),
            voice_leading_style: self.inner.voice_leading_style(),
            octave_mode: self.inner.octave_mode(),
            humanize_config: HumanizeConfig::default(),
            scale_mode: self.inner.scale_mode(),
            interchange_enabled: self.inner.interchange_enabled(),
            borrowing_range: self.inner.borrowing_range(),
            is_builtin: false,
        };

        self.presets.add_custom(preset);
        Ok(())
    }

    /// Delete a custom preset by name.
    pub fn delete_preset(&mut self, name: &str) -> Result<(), JsValue> {
        let custom = self.presets.custom_presets();
        if let Some(idx) = custom.iter().position(|p| p.name == name) {
            self.presets.remove_custom(idx);
            Ok(())
        } else {
            Err(JsValue::from_str(&format!(
                "Custom preset not found: {}",
                name
            )))
        }
    }

    // === Next-Note Suggestion Overlay ===

    /// Compute ranked note suggestions based on current engine state.
    ///
    /// Returns a JSON-serialized array of `{note: u8, score: f32}` objects,
    /// limited to the top 12 suggestions. This is a visual overlay -- the
    /// suggestions are never played audibly.
    pub fn get_suggestions(&self) -> Result<JsValue, JsValue> {
        use contrapunk::harmony::suggestion::rank_candidates;

        let snapshot = self.inner.suggestion_snapshot();
        let ranked = rank_candidates(&snapshot, &self.suggestion_config);
        let top: Vec<SuggestionScoreJs> = ranked
            .iter()
            .take(12)
            .map(|&(note, score)| SuggestionScoreJs { note, score })
            .collect();

        serde_wasm_bindgen::to_value(&top)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Set a single suggestion weight by term name.
    ///
    /// Valid term names: chord_tone, scale_tone, dissonance, proximity,
    /// contour, leap_recovery, repetition, next_chord_prep, leading_tone,
    /// narmour, tessitura.
    pub fn set_suggestion_weight(&mut self, term: &str, value: f32) -> Result<(), JsValue> {
        if self.suggestion_config.set_weight(term, value) {
            Ok(())
        } else {
            Err(JsValue::from_str(&format!(
                "Unknown suggestion term: {}",
                term
            )))
        }
    }

    /// Get all current suggestion weights as a JSON object.
    pub fn get_suggestion_weights(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.suggestion_config)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Reset suggestion weights to Bach chorale calibrated defaults.
    pub fn reset_suggestion_weights(&mut self) {
        self.suggestion_config = contrapunk::harmony::suggestion::SuggestionConfig::default();
    }
}

// === WASM-exported GuitarInput wrapper ===

use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig, MidiEvent};

#[wasm_bindgen]
pub struct WasmGuitarInput {
    inner: GuitarInput,
    frame_count: u64,
}

#[wasm_bindgen]
impl WasmGuitarInput {
    /// Create a new GuitarInput DSP pipeline with the given sample rate and buffer size.
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: usize, buffer_size: usize) -> Self {
        let config = GuitarInputConfig {
            buffer_size,
            sample_rate,
            onset_threshold: 0.015,             // match demo
            string_confidence_min: 0.4,         // match demo
            cooldown_samples: sample_rate / 10, // 100ms at actual sample rate
            ..GuitarInputConfig::default()
        };
        console_log!(
            "[wasm-guitar] Created: sr={} buf={} onset={} clarity={} gain={}",
            sample_rate,
            buffer_size,
            config.onset_threshold,
            config.min_clarity,
            config.input_gain
        );
        Self {
            inner: GuitarInput::new(config),
            frame_count: 0,
        }
    }

    /// Process an audio block and return MIDI events as a JSON string.
    /// Input: Float32Array of mono audio samples.
    /// Output: JSON array of event objects.
    pub fn process_block(&mut self, samples: &[f32]) -> String {
        let events = self.inner.process_block(samples);

        self.frame_count += 1;

        // Log internal DSP state every 20 frames
        if self.frame_count % 20 == 0 {
            let rms: f32 =
                (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
            let state = match self.inner.note_state_name() {
                0 => "Idle",
                1 => "Attack",
                2 => "Sustain",
                3 => "Decay",
                _ => "?",
            };
            let pitch_str = match self.inner.last_debug_pitch {
                Some((f, c)) => format!("freq={:.1} clr={:.2}", f, c),
                None => "None".to_string(),
            };
            console_log!(
                "[analyze] rms={:.4} prevRms={:.4} onset={} (rms={} flux={} f={:.3}) pitch={} state={}",
                rms, self.inner.prev_rms(),
                self.inner.last_debug_onset,
                self.inner.last_debug_rms_onset,
                self.inner.last_debug_flux_onset,
                self.inner.last_debug_flux,
                pitch_str,
                state
            );
        }
        if !events.is_empty() {
            console_log!(
                "[wasm-guitar] {} events at frame {}",
                events.len(),
                self.frame_count
            );
        }

        if events.is_empty() {
            return "[]".to_string();
        }

        let json_events: Vec<String> = events
            .iter()
            .map(|e| match e {
                MidiEvent::NoteOn {
                    channel,
                    note,
                    velocity,
                } => format!(
                    r#"{{"type":"note_on","channel":{},"note":{},"velocity":{}}}"#,
                    channel, note, velocity
                ),
                MidiEvent::NoteOff {
                    channel,
                    note,
                    velocity,
                } => format!(
                    r#"{{"type":"note_off","channel":{},"note":{},"velocity":{}}}"#,
                    channel, note, velocity
                ),
                MidiEvent::PitchBend { channel, cents } => format!(
                    r#"{{"type":"pitch_bend","channel":{},"cents":{}}}"#,
                    channel, cents
                ),
                MidiEvent::MidiPitchBend { channel, value } => format!(
                    r#"{{"type":"midi_pitch_bend","channel":{},"value":{}}}"#,
                    channel, value
                ),
                MidiEvent::ChannelPressure { channel, pressure } => format!(
                    r#"{{"type":"channel_pressure","channel":{},"pressure":{}}}"#,
                    channel, pressure
                ),
                MidiEvent::CC {
                    channel,
                    controller,
                    value,
                } => format!(
                    r#"{{"type":"cc","channel":{},"controller":{},"value":{}}}"#,
                    channel, controller, value
                ),
                MidiEvent::VibratoStatus {
                    active,
                    rate_hz,
                    depth_cents,
                } => format!(
                    r#"{{"type":"vibrato","active":{},"rate_hz":{},"depth_cents":{}}}"#,
                    active, rate_hz, depth_cents
                ),
            })
            .collect();

        format!("[{}]", json_events.join(","))
    }

    /// Set onset threshold (default 0.015).
    pub fn set_onset_threshold(&mut self, val: f32) {
        self.inner.config_mut().onset_threshold = val;
    }

    /// Set string confidence minimum (default 0.4).
    pub fn set_string_confidence(&mut self, val: f32) {
        self.inner.config_mut().string_confidence_min = val;
    }

    /// Set input gain (default 1.0).
    pub fn set_input_gain(&mut self, val: f32) {
        self.inner.config_mut().input_gain = val;
    }

    /// Enable/disable pitch bend detection.
    pub fn set_bends_enabled(&mut self, val: bool) {
        self.inner.config_mut().bends_enabled = val;
    }

    /// Enable/disable legato detection.
    pub fn set_legato_enabled(&mut self, val: bool) {
        self.inner.config_mut().legato_enabled = val;
    }

    /// Enable/disable slide detection.
    pub fn set_slides_enabled(&mut self, val: bool) {
        self.inner.config_mut().slides_enabled = val;
    }

    /// Enable/disable vibrato detection.
    pub fn set_vibrato_enabled(&mut self, val: bool) {
        self.inner.config_mut().vibrato_detection = val;
    }

    /// Free resources.
    pub fn free(self) {}
}

/// Convert a MIDI note number (0-127) to its note name (e.g. 60 -> "C4").
#[wasm_bindgen]
pub fn midi_to_name(midi: u8) -> String {
    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (midi as i8 / 12) - 1;
    let name_idx = (midi % 12) as usize;
    format!("{}{}", note_names[name_idx], octave)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every species name that the UI sends must parse back to the matching
    /// `CounterpointSpecies` variant. This is the bridge-layer smoke test:
    /// if the UI string contract drifts, this catches it at build time.
    #[test]
    fn test_counterpoint_species_roundtrip() {
        let cases = [
            ("Species1", CounterpointSpecies::Species1),
            ("Species2", CounterpointSpecies::Species2),
            ("Species3", CounterpointSpecies::Species3),
            ("Species4", CounterpointSpecies::Species4),
        ];
        for (name, expected) in cases {
            let parsed = parse_counterpoint_species(name)
                .unwrap_or_else(|_| panic!("should parse {}", name));
            assert_eq!(parsed, expected);
            // And the reverse direction: enum -> string the UI expects back.
            assert_eq!(counterpoint_species_to_string(expected), name);
        }
    }

    #[test]
    fn test_counterpoint_strictness_roundtrip() {
        let cases = [
            ("Relaxed", CounterpointStrictness::Relaxed),
            ("Strict", CounterpointStrictness::Strict),
        ];
        for (name, expected) in cases {
            let parsed = parse_counterpoint_strictness(name)
                .unwrap_or_else(|_| panic!("should parse {}", name));
            assert_eq!(parsed, expected);
            assert_eq!(counterpoint_strictness_to_string(expected), name);
        }
    }

    // The error path constructs `JsValue::from_str`, which panics on
    // non-wasm32 targets. Only compile these when testing WASM directly
    // (e.g. via wasm-bindgen-test).
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_counterpoint_species_rejects_unknown() {
        assert!(parse_counterpoint_species("Species5").is_err());
        assert!(parse_counterpoint_species("").is_err());
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_counterpoint_strictness_rejects_unknown() {
        assert!(parse_counterpoint_strictness("Loose").is_err());
        assert!(parse_counterpoint_strictness("").is_err());
    }

    // === Humanizer / Beat Clock / Metronome integration ===
    //
    // These tests poke at the `Engine` through its wasm-bindgen-exported
    // methods that don't return `JsValue` (the pure-Rust subset). Methods
    // like `tick()` that serialize into `JsValue` are only exercised under
    // `target_arch = "wasm32"` because `JsValue::from_str` panics natively.

    #[test]
    fn test_humanizer_defaults() {
        // A freshly-constructed Engine should have humanization disabled
        // and a default 120 BPM clock, and the metronome off.
        let e = Engine::new();
        assert!(!e.is_humanize_enabled());
        assert!(!e.is_metronome_enabled());
        assert!((e.bpm() - 120.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_bpm_and_time_signature() {
        let mut e = Engine::new();
        e.set_bpm(140.0);
        assert!((e.bpm() - 140.0).abs() < f64::EPSILON);

        e.set_time_signature(3, 4);
        assert_eq!(e.humanizer.config().beats_per_bar, 3);
        assert_eq!(e.humanizer.config().beat_unit, 4);
    }

    #[test]
    fn test_humanize_toggles() {
        let mut e = Engine::new();
        e.set_humanize_enabled(true);
        assert!(e.is_humanize_enabled());
        e.set_metronome_enabled(true);
        assert!(e.is_metronome_enabled());
        e.set_metronome_enabled(false);
        assert!(!e.is_metronome_enabled());
    }

    /// Drive the beat clock forward via `Humanizer::tick` directly (the
    /// WASM-exported `Engine::tick` wraps a JsValue serialization which
    /// panics on native test targets). This verifies beat position advances.
    #[test]
    fn test_beat_clock_advances() {
        let mut e = Engine::new();
        e.set_bpm(120.0);
        // Simulate the lazy start that Engine::tick normally performs.
        e.humanizer.clock_mut().start(0.0);
        // At 120 BPM: 1 beat = 500ms.
        e.humanizer.tick(500.0);
        let pos = e.beat_position();
        assert!(
            pos > 0.9 && pos < 1.1,
            "expected ~1.0 beat position at 500ms/120bpm, got {}",
            pos
        );
    }

    /// Crossing a beat boundary with the metronome enabled should emit a
    /// click. The `Metronome::generate_click` path is verified end-to-end
    /// on native so we know the bytes are well-formed MIDI.
    #[test]
    fn test_metronome_click_bytes_on_beat_crossing() {
        let mut e = Engine::new();
        e.set_bpm(120.0);
        e.set_metronome_enabled(true);
        e.humanizer.clock_mut().start(0.0);

        // Step across a beat boundary.
        e.humanizer.tick(0.0);
        e.humanizer.tick(600.0);
        let crossed = e.humanizer.clock().beat_crossed();
        assert_eq!(crossed, Some(1), "should have crossed into beat 1");

        // Generate a click for that beat; bytes should be a 3-byte NoteOn
        // on channel 10 (status 0x99) with a woodblock note.
        let bytes = e.metronome.generate_click(1);
        assert_eq!(bytes.len(), 3);
        assert_eq!(bytes[0] & 0xF0, 0x90, "should be a NoteOn status nibble");
        assert_eq!(bytes[0] & 0x0F, 9, "should be channel 10 (index 9)");
        assert_eq!(bytes[1], 77, "off-beat click should be low woodblock (77)");
    }

    /// Bridge test for Counterpoint Species 2-4: once we push a beat-phase
    /// value into the inner HarmonyEngine, subsequent harmonize calls
    /// should behave differently than Species 1 (which ignores beat phase).
    /// We only check that the setter works without panicking; the species
    /// behaviour itself is covered by harmony::counterpoint tests.
    #[test]
    fn test_counterpoint_beat_phase_plumbing() {
        let mut e = Engine::new();
        // Explicit None / Some round-trip.
        e.set_counterpoint_beat_phase(Some(2.5));
        e.set_counterpoint_beat_phase(None);
        // If the above didn't panic, the wiring is intact.
    }
}
