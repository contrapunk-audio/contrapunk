//! String <-> enum conversions for the WASM-bindgen API surface.
//!
//! The Engine exposes `set_key("C")`, `set_mode("DiatonicThirds")` etc.
//! to JavaScript. This module is the one place where those wire-protocol
//! strings live. Drift between this module and the UI store's typed
//! enums in `ui/src/lib/stores/engine.svelte.ts` is the most common
//! bridge-layer bug; the roundtrip tests in `lib.rs::tests` are the
//! contract guard.
//!
//! Extracted from `lib.rs` in v1.2.x Phase 1 to drop the WASM bridge's
//! line count (was ~947, this module owns ~290 of those) and isolate
//! the boilerplate from the actual harmony / DSP wiring.

use contrapunk::harmony::{
    CounterpointSpecies, CounterpointStrictness, HarmonyMode, Key, OctaveMode, ScaleMode,
    VoiceLeadingStyle,
};
use wasm_bindgen::prelude::*;

// === String -> enum parsers ===

pub(crate) fn parse_key(s: &str) -> Result<Key, JsValue> {
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

pub(crate) fn parse_mode(s: &str) -> Result<HarmonyMode, JsValue> {
    match s {
        "PassThrough" => Ok(HarmonyMode::PassThrough),
        "DiatonicThirds" => Ok(HarmonyMode::DiatonicThirds),
        "DiatonicFourths" => Ok(HarmonyMode::DiatonicFourths),
        "RandomBelow" => Ok(HarmonyMode::ContraryMotion),
        "RandomBelowNoSeconds" => Ok(HarmonyMode::StrictCounterpoint),
        "ContraryMotion" => Ok(HarmonyMode::ContraryMotion),
        "StrictCounterpoint" => Ok(HarmonyMode::StrictCounterpoint),
        "BarryHarris" => Ok(HarmonyMode::BarryHarris),
        "FunctionalHarmony" => Ok(HarmonyMode::FunctionalHarmony),
        "BachChorale" => Ok(HarmonyMode::BachChorale),
        "ExplicitIntervals" => Ok(HarmonyMode::ExplicitIntervals),
        _ => Err(JsValue::from_str(&format!("Unknown mode: {}", s))),
    }
}

pub(crate) fn parse_scale_mode(s: &str) -> Result<ScaleMode, JsValue> {
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

pub(crate) fn parse_octave_mode(s: &str) -> Result<OctaveMode, JsValue> {
    match s {
        "None" => Ok(OctaveMode::None),
        "Spread" => Ok(OctaveMode::Spread),
        "BassTrebleSplit" => Ok(OctaveMode::BassTrebleSplit),
        "Mirror" => Ok(OctaveMode::Mirror),
        _ => Err(JsValue::from_str(&format!("Unknown octave mode: {}", s))),
    }
}

pub(crate) fn parse_voice_leading_style(s: &str) -> Result<VoiceLeadingStyle, JsValue> {
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

pub(crate) fn parse_counterpoint_species(s: &str) -> Result<CounterpointSpecies, JsValue> {
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

pub(crate) fn parse_counterpoint_strictness(s: &str) -> Result<CounterpointStrictness, JsValue> {
    match s {
        "Relaxed" | "relaxed" => Ok(CounterpointStrictness::Relaxed),
        "Strict" | "strict" => Ok(CounterpointStrictness::Strict),
        _ => Err(JsValue::from_str(&format!(
            "Unknown counterpoint strictness: {}",
            s
        ))),
    }
}

// === Enum -> string ===

pub(crate) fn counterpoint_species_to_string(s: CounterpointSpecies) -> &'static str {
    match s {
        CounterpointSpecies::Species1 => "Species1",
        CounterpointSpecies::Species2 => "Species2",
        CounterpointSpecies::Species3 => "Species3",
        CounterpointSpecies::Species4 => "Species4",
    }
}

pub(crate) fn counterpoint_strictness_to_string(s: CounterpointStrictness) -> &'static str {
    match s {
        CounterpointStrictness::Relaxed => "Relaxed",
        CounterpointStrictness::Strict => "Strict",
    }
}

pub(crate) fn key_to_string(key: Key) -> &'static str {
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

pub(crate) fn mode_to_string(mode: HarmonyMode) -> &'static str {
    match mode {
        HarmonyMode::PassThrough => "PassThrough",
        HarmonyMode::DiatonicThirds => "DiatonicThirds",
        HarmonyMode::DiatonicFourths => "DiatonicFourths",
        HarmonyMode::ContraryMotion => "ContraryMotion",
        HarmonyMode::StrictCounterpoint => "StrictCounterpoint",
        HarmonyMode::BarryHarris => "BarryHarris",
        HarmonyMode::FunctionalHarmony => "FunctionalHarmony",
        HarmonyMode::BachChorale => "BachChorale",
        HarmonyMode::ExplicitIntervals => "ExplicitIntervals",
    }
}

pub(crate) fn scale_mode_to_string(mode: ScaleMode) -> &'static str {
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

pub(crate) fn octave_mode_to_string(mode: OctaveMode) -> &'static str {
    match mode {
        OctaveMode::None => "None",
        OctaveMode::Spread => "Spread",
        OctaveMode::BassTrebleSplit => "BassTrebleSplit",
        OctaveMode::Mirror => "Mirror",
    }
}

pub(crate) fn voice_leading_style_to_string(style: VoiceLeadingStyle) -> &'static str {
    match style {
        VoiceLeadingStyle::Free => "Free",
        VoiceLeadingStyle::Palestrina => "Palestrina",
        VoiceLeadingStyle::BachChorale => "BachChorale",
        VoiceLeadingStyle::Jazz => "Jazz",
    }
}
