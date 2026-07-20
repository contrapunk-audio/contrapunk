//! WASM Bridge for Contrapunk Harmony Engine
//!
//! Exposes the core harmony engine to JavaScript/TypeScript via wasm-bindgen.
//! This crate wraps `contrapunk::harmony::HarmonyEngine` and provides
//! string-based APIs suitable for the Svelte UI adapter layer.

use wasm_bindgen::prelude::*;

use std::collections::HashSet;

use contrapunk::chord::chord_display_with_analysis;
use contrapunk::harmony::{ExplicitIntervalMap, HarmonyEngine, HarmonyMode, Key};
use contrapunk::preset::PresetManager;

mod companion;
mod enum_strings;
#[allow(unused_imports)]
pub use companion::CompanionWasm;
use enum_strings::*;

/// Log to browser console from Rust WASM. Debug-only — release builds
/// expand to a no-op so the per-block guitar-DSP traces at lines 716+
/// don't burn frame budget or flood DevTools when users have the app
/// open. CONCERNS.md flagged this as a measurable jank source under
/// DevTools-open conditions; gating at the macro level keeps all
/// existing call sites working transparently. Use `web_sys::console::
/// error_1` directly for errors that must survive release builds.
#[cfg(debug_assertions)]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()))
}

#[cfg(not(debug_assertions))]
macro_rules! console_log {
    ($($t:tt)*) => {
        ()
    };
}

// Initialize panic hook for better error messages in the browser console.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// === Helpers extracted to `enum_strings` module ===
// The 290 lines of string<->enum boilerplate that used to live here
// moved to wasm/src/enum_strings.rs in v1.2.x Phase 1 to drop this
// file's entropy score. The roundtrip tests in mod tests below still
// pin the wire-protocol contract.

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
    counterpoint_species: &'static str,
    counterpoint_strictness: &'static str,
    explicit_interval_map: ExplicitIntervalMap,
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

    /// Continuous octave-spread coefficient. Range [0.0, 1.0]; 0 = no
    /// displacement, 1 = full-octave (legacy) per-voice displacement.
    pub fn set_octave_intensity(&mut self, amount: f32) {
        self.inner.set_octave_intensity(amount);
        self.clear_notes();
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

    /// Replace the source-degree-to-semitone explicit interval map.
    pub fn set_explicit_interval_map(&mut self, json: &str) -> Result<(), JsValue> {
        let map: ExplicitIntervalMap = serde_json::from_str(json).map_err(|error| {
            JsValue::from_str(&format!("Invalid explicit interval map: {error}"))
        })?;
        self.inner
            .set_explicit_interval_map(map)
            .map_err(|error| JsValue::from_str(&error))?;
        self.clear_notes();
        Ok(())
    }

    /// Set the counterpoint species (`"Species1"` through `"Species4"`).
    ///
    /// Only active when the harmony mode is `StrictCounterpoint`. Species 2-4
    /// require beat-phase input via `set_counterpoint_beat_phase`; without
    /// it they fall back to Species 1 behavior.
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

    /// Returns whether bass-register suppression is active. Issue #100.
    pub fn suppress_bass_register(&self) -> bool {
        self.inner.suppress_bass_register()
    }

    /// Enable or disable bass-register suppression. When on, input
    /// notes below the threshold pass through without producing
    /// harmony — for users who play the bass line themselves.
    pub fn set_suppress_bass_register(&mut self, enabled: bool) {
        self.inner.set_suppress_bass_register(enabled);
        self.clear_notes();
    }

    /// Returns the bass-register threshold MIDI note number.
    pub fn bass_register_threshold(&self) -> u8 {
        self.inner.bass_register_threshold()
    }

    /// Sets the bass-register threshold MIDI note (notes below pass
    /// through when `suppress_bass_register` is true). Clamped to
    /// 0..=127.
    pub fn set_bass_register_threshold(&mut self, midi: u8) {
        self.inner.set_bass_register_threshold(midi);
        self.clear_notes();
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
            explicit_interval_map: self.inner.explicit_interval_map().clone(),
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

        // Detect chord from currently-sounding notes (input + harmony).
        // The Tauri router thread runs the same call when populating
        // its `chord_name` mutex; running it here keeps the browser /
        // WASM consumers (Astro embed widgets in particular) feature-
        // parity for chord readouts without going through the desktop
        // audio path.
        let chord_name = {
            let all_sounding: HashSet<u8> = self
                .last_input_notes
                .iter()
                .chain(self.last_harmony_notes.iter())
                .copied()
                .collect();
            if all_sounding.is_empty() {
                String::new()
            } else {
                let key_tonic = Some(self.inner.key().semitones_from_c());
                chord_display_with_analysis(&all_sounding, key_tonic)
            }
        };

        let state = NoteStateJs {
            input_notes: self.last_input_notes.clone(),
            harmony_notes: self.last_harmony_notes.clone(),
            borrowed_notes: Vec::new(), // Still populated by routing layer; WASM consumers don't expose it yet
            chord_name,
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

        // Log internal DSP state every 20 frames — gated on debug builds
        // so release WASM doesn't waste audio-block budget on RMS, state-
        // lookup, and pitch-formatting computations whose output is then
        // thrown away by the no-op console_log! macro in release.
        // CONCERNS.md flagged the per-frame log path as a measurable jank
        // source; this gate removes both the log AND its computation.
        if cfg!(debug_assertions) && self.frame_count % 20 == 0 {
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
        if cfg!(debug_assertions) && !events.is_empty() {
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
    use contrapunk::harmony::{CounterpointSpecies, CounterpointStrictness};

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
