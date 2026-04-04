//! WASM Bridge for Contrapunk Harmony Engine
//!
//! Exposes the core harmony engine to JavaScript/TypeScript via wasm-bindgen.
//! This crate wraps `contrapunk::harmony::HarmonyEngine` and provides
//! string-based APIs suitable for the Svelte UI adapter layer.

use wasm_bindgen::prelude::*;

use contrapunk::harmony::VoiceLeadingStyle;
use contrapunk::harmony::{HarmonyEngine, HarmonyMode, Key, OctaveMode, ScaleMode};
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
        _ => Err(JsValue::from_str(&format!("Unknown mode: {}", s))),
    }
}

fn parse_scale_mode(s: &str) -> Result<ScaleMode, JsValue> {
    match s {
        // Church
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
        // Exotic
        "DoubleHarmonic" => Ok(ScaleMode::DoubleHarmonic),
        "HungarianMinor" => Ok(ScaleMode::HungarianMinor),
        "Enigmatic" => Ok(ScaleMode::Enigmatic),
        "NeapolitanMinor" => Ok(ScaleMode::NeapolitanMinor),
        "NeapolitanMajor" => Ok(ScaleMode::NeapolitanMajor),
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
    }
}

fn scale_mode_to_string(mode: ScaleMode) -> &'static str {
    match mode {
        ScaleMode::Ionian => "Ionian",
        ScaleMode::Dorian => "Dorian",
        ScaleMode::Phrygian => "Phrygian",
        ScaleMode::Lydian => "Lydian",
        ScaleMode::Mixolydian => "Mixolydian",
        ScaleMode::Aeolian => "Aeolian",
        ScaleMode::Locrian => "Locrian",
        ScaleMode::HarmonicMinor => "HarmonicMinor",
        ScaleMode::LocrianNat6 => "LocrianNat6",
        ScaleMode::IonianAug => "IonianAug",
        ScaleMode::DorianSharp4 => "DorianSharp4",
        ScaleMode::PhrygianDominant => "PhrygianDominant",
        ScaleMode::LydianSharp2 => "LydianSharp2",
        ScaleMode::SuperLocrianDim => "SuperLocrianDim",
        ScaleMode::MelodicMinor => "MelodicMinor",
        ScaleMode::DorianFlat2 => "DorianFlat2",
        ScaleMode::LydianAug => "LydianAug",
        ScaleMode::LydianDominant => "LydianDominant",
        ScaleMode::MixolydianFlat6 => "MixolydianFlat6",
        ScaleMode::LocrianNat2 => "LocrianNat2",
        ScaleMode::SuperLocrian => "SuperLocrian",
        ScaleMode::DoubleHarmonic => "DoubleHarmonic",
        ScaleMode::HungarianMinor => "HungarianMinor",
        ScaleMode::Enigmatic => "Enigmatic",
        ScaleMode::NeapolitanMinor => "NeapolitanMinor",
        ScaleMode::NeapolitanMajor => "NeapolitanMajor",
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

#[wasm_bindgen]
pub struct Engine {
    inner: HarmonyEngine,
    presets: PresetManager,
    /// Track notes that were played through note_on for note state reporting
    last_input_notes: Vec<u8>,
    last_harmony_notes: Vec<u8>,
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
            onset_threshold: 0.015,         // match demo
            string_confidence_min: 0.4,      // match demo
            cooldown_samples: sample_rate / 10, // 100ms at actual sample rate
            ..GuitarInputConfig::default()
        };
        console_log!(
            "[wasm-guitar] Created: sr={} buf={} onset={} clarity={} gain={}",
            sample_rate, buffer_size, config.onset_threshold, config.min_clarity, config.input_gain
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
            let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
            let state = match self.inner.note_state_name() {
                0 => "Idle", 1 => "Attack", 2 => "Sustain", 3 => "Decay", _ => "?",
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
            console_log!("[wasm-guitar] {} events at frame {}", events.len(), self.frame_count);
        }

        if events.is_empty() {
            return "[]".to_string();
        }

        let json_events: Vec<String> = events.iter().map(|e| match e {
            MidiEvent::NoteOn { channel, note, velocity } =>
                format!(r#"{{"type":"note_on","channel":{},"note":{},"velocity":{}}}"#, channel, note, velocity),
            MidiEvent::NoteOff { channel, note, velocity } =>
                format!(r#"{{"type":"note_off","channel":{},"note":{},"velocity":{}}}"#, channel, note, velocity),
            MidiEvent::PitchBend { channel, cents } =>
                format!(r#"{{"type":"pitch_bend","channel":{},"cents":{}}}"#, channel, cents),
            MidiEvent::MidiPitchBend { channel, value } =>
                format!(r#"{{"type":"midi_pitch_bend","channel":{},"value":{}}}"#, channel, value),
            MidiEvent::ChannelPressure { channel, pressure } =>
                format!(r#"{{"type":"channel_pressure","channel":{},"pressure":{}}}"#, channel, pressure),
            MidiEvent::CC { channel, controller, value } =>
                format!(r#"{{"type":"cc","channel":{},"controller":{},"value":{}}}"#, channel, controller, value),
            MidiEvent::VibratoStatus { active, rate_hz, depth_cents } =>
                format!(r#"{{"type":"vibrato","active":{},"rate_hz":{},"depth_cents":{}}}"#, active, rate_hz, depth_cents),
        }).collect();

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

// === Guitar Input DSP (full pipeline via WASM) ===

use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig, MidiEvent};

#[wasm_bindgen]
pub struct WasmGuitarInput {
    inner: GuitarInput,
    pb_range: u8,
}

#[wasm_bindgen]
impl WasmGuitarInput {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: usize, buffer_size: usize) -> Self {
        let config = GuitarInputConfig {
            sample_rate,
            buffer_size,
            ..GuitarInputConfig::default()
        };
        Self { inner: GuitarInput::new(config), pb_range: 2 }
    }

    /// Process audio block, returns JSON array of MIDI events.
    pub fn process_block(&mut self, audio: &[f32]) -> String {
        let events = self.inner.process_block(audio);
        let json_events: Vec<serde_json::Value> = events.iter().filter_map(|e| {
            match e {
                MidiEvent::NoteOn { channel, note, velocity } => Some(serde_json::json!({
                    "type": "note_on", "channel": *channel, "note": *note, "velocity": *velocity
                })),
                MidiEvent::NoteOff { channel, note, velocity } => Some(serde_json::json!({
                    "type": "note_off", "channel": *channel, "note": *note, "velocity": *velocity
                })),
                MidiEvent::PitchBend { channel, cents } => Some(serde_json::json!({
                    "type": "pitch_bend", "channel": *channel, "cents": *cents
                })),
                MidiEvent::CC { channel, controller, value } => Some(serde_json::json!({
                    "type": "cc", "channel": *channel, "controller": *controller, "value": *value
                })),
                MidiEvent::ChannelPressure { channel, pressure } => Some(serde_json::json!({
                    "type": "channel_pressure", "channel": *channel, "pressure": *pressure
                })),
                MidiEvent::VibratoStatus { active, rate_hz, depth_cents } => Some(serde_json::json!({
                    "type": "vibrato", "active": *active, "rate_hz": *rate_hz, "depth_cents": *depth_cents
                })),
                MidiEvent::MidiPitchBend { .. } => None,
            }
        }).collect();
        serde_json::to_string(&json_events).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn set_bends_enabled(&mut self, v: bool) { self.inner.config_mut().bends_enabled = v; }
    pub fn set_legato_enabled(&mut self, v: bool) { self.inner.config_mut().legato_enabled = v; }
    pub fn set_slides_enabled(&mut self, v: bool) { self.inner.config_mut().slides_enabled = v; }
    pub fn set_vibrato_enabled(&mut self, v: bool) { self.inner.config_mut().vibrato_detection = v; }
    pub fn set_input_gain(&mut self, v: f32) { self.inner.config_mut().input_gain = v; }
    pub fn set_onset_threshold(&mut self, v: f32) { self.inner.config_mut().onset_threshold = v; }
    pub fn set_string_confidence(&mut self, v: f32) { self.inner.config_mut().string_confidence_min = v; }
}
