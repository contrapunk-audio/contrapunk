pub mod builtins;
pub mod storage;

use serde::{Serialize, Deserialize};
use crate::harmony::{Key, HarmonyMode, OctaveMode, VoiceLeadingStyle};
use crate::humanize::HumanizeConfig;

/// A complete musical style preset bundling harmony, voice leading,
/// humanization, and octave settings into a selectable style.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StylePreset {
    pub name: String,
    /// Character persona name
    pub persona: String,
    /// Genre subtitle
    pub genre: String,
    pub harmony_mode: HarmonyMode,
    pub key: Key,
    pub voice_leading_enabled: bool,
    pub voice_leading_style: VoiceLeadingStyle,
    pub octave_mode: OctaveMode,
    pub humanize_config: HumanizeConfig,
    #[serde(skip)]
    pub is_builtin: bool,
}

/// Manages built-in and custom presets.
pub struct PresetManager {
    builtins: Vec<StylePreset>,
    custom: Vec<StylePreset>,
    /// Index into combined list (builtins first, then custom).
    active_index: Option<usize>,
}

impl PresetManager {
    /// Creates a new PresetManager with all built-in presets loaded.
    pub fn new() -> Self {
        Self {
            builtins: builtins::all(),
            custom: Vec::new(),
            active_index: None,
        }
    }

    /// Returns all presets (builtins first, then custom).
    pub fn all_presets(&self) -> Vec<&StylePreset> {
        self.builtins.iter().chain(self.custom.iter()).collect()
    }

    /// Returns the currently active preset, if any.
    pub fn active(&self) -> Option<&StylePreset> {
        let idx = self.active_index?;
        let all = self.all_presets();
        all.get(idx).copied()
    }

    /// Sets the active preset by index into the combined list.
    pub fn set_active(&mut self, index: usize) {
        let total = self.builtins.len() + self.custom.len();
        if index < total {
            self.active_index = Some(index);
        }
    }

    /// Adds a custom preset.
    pub fn add_custom(&mut self, preset: StylePreset) {
        self.custom.push(preset);
    }

    /// Removes a custom preset by index within the custom list.
    pub fn remove_custom(&mut self, index: usize) {
        if index < self.custom.len() {
            self.custom.remove(index);
            // Reset active if it pointed to a removed or shifted preset
            if let Some(active) = self.active_index {
                let builtin_len = self.builtins.len();
                if active >= builtin_len + index {
                    self.active_index = None;
                }
            }
        }
    }

    /// Returns a slice of custom presets.
    pub fn custom_presets(&self) -> &[StylePreset] {
        &self.custom
    }

    /// Replaces all custom presets (used when loading from storage).
    pub fn set_custom_presets(&mut self, presets: Vec<StylePreset>) {
        self.custom = presets;
    }
}
