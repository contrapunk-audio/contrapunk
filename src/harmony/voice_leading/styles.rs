//! Voice leading style presets with hard vs soft constraint distinction.

use serde::{Serialize, Deserialize};

/// Voice leading style presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceLeadingStyle {
    /// Renaissance polyphony — strictest rules
    Palestrina,
    /// Baroque four-part writing
    BachChorale,
    /// Extended harmony, relaxed parallels
    Jazz,
    /// Minimal constraints
    #[default]
    Free,
}

impl VoiceLeadingStyle {
    /// Returns all styles for menu display.
    pub fn all() -> &'static [VoiceLeadingStyle] {
        &[
            VoiceLeadingStyle::Palestrina,
            VoiceLeadingStyle::BachChorale,
            VoiceLeadingStyle::Jazz,
            VoiceLeadingStyle::Free,
        ]
    }

    /// Returns a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            VoiceLeadingStyle::Palestrina => "Palestrina (Renaissance polyphony)",
            VoiceLeadingStyle::BachChorale => "Bach Chorale (Baroque four-part)",
            VoiceLeadingStyle::Jazz => "Jazz (extended harmony)",
            VoiceLeadingStyle::Free => "Free (minimal constraints)",
        }
    }
}

impl std::fmt::Display for VoiceLeadingStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Style rules defining hard constraints (reject) and soft penalties (score).
#[derive(Clone, Debug)]
pub struct StyleRules {
    /// If true, any parallel fifths cause outright rejection
    pub hard_reject_parallel_fifths: bool,
    /// If true, any parallel octaves cause outright rejection
    pub hard_reject_parallel_octaves: bool,
    /// Penalty for parallel fifths (when not hard-rejected)
    pub parallel_fifths_penalty: i32,
    /// Penalty for parallel octaves (when not hard-rejected)
    pub parallel_octaves_penalty: i32,
    /// Penalty per voice-crossing pair
    pub voice_crossing_penalty: i32,
    /// Bonus for stepwise motion (1-2 semitones)
    pub stepwise_bonus: i32,
    /// Bonus for retaining common tones between chords
    pub common_tone_bonus: i32,
    /// Penalty per semitone of leap (negative)
    pub leap_penalty_per_semitone: i32,
    /// Maximum allowed leap in semitones
    pub max_leap_semitones: u8,
    /// Penalty when all harmony voices move in parallel
    pub all_parallel_motion_penalty: i32,
    /// Penalty per spacing violation
    pub spacing_violation_penalty: i32,
    /// Bonus per semitone of inter-voice spread (positive = prefer wide, negative = prefer tight)
    pub spread_preference: i32,
    /// Bonus when a voice moves opposite direction to melody
    pub contrary_motion_bonus: i32,
}

impl StyleRules {
    /// Returns the style rules for a given voice leading style.
    pub fn for_style(style: VoiceLeadingStyle) -> StyleRules {
        match style {
            // Palestrina: Renaissance polyphony — extremely tight, stepwise,
            // no leaps beyond a 4th, voices clustered close together,
            // strong contrary motion preference. Should sound like a
            // choir singing in close harmony with smooth flowing lines.
            VoiceLeadingStyle::Palestrina => StyleRules {
                hard_reject_parallel_fifths: true,
                hard_reject_parallel_octaves: true,
                parallel_fifths_penalty: -200,
                parallel_octaves_penalty: -200,
                voice_crossing_penalty: -150,
                stepwise_bonus: 60,
                common_tone_bonus: 45,
                leap_penalty_per_semitone: -15,
                max_leap_semitones: 5,
                all_parallel_motion_penalty: -120,
                spacing_violation_penalty: -200,
                spread_preference: -4,
                contrary_motion_bonus: 40,
            },
            // Bach Chorale: Common tones are king, moderate stepwise,
            // allows leaps up to an octave, bass voice can leap freely.
            // Should sound like hymn harmonization — smooth but with
            // purposeful bass movement and held inner voices.
            VoiceLeadingStyle::BachChorale => StyleRules {
                hard_reject_parallel_fifths: true,
                hard_reject_parallel_octaves: true,
                parallel_fifths_penalty: -100,
                parallel_octaves_penalty: -100,
                voice_crossing_penalty: -80,
                stepwise_bonus: 25,
                common_tone_bonus: 70,
                leap_penalty_per_semitone: -4,
                max_leap_semitones: 12,
                all_parallel_motion_penalty: -60,
                spacing_violation_penalty: -80,
                spread_preference: -1,
                contrary_motion_bonus: 20,
            },
            // Jazz: Wide spread voicings, leaps are fine, parallel
            // motion is fine, voices spread across the keyboard.
            // Should sound open and spacious, like a jazz piano
            // with drop-2/drop-3 voicings.
            VoiceLeadingStyle::Jazz => StyleRules {
                hard_reject_parallel_fifths: false,
                hard_reject_parallel_octaves: false,
                parallel_fifths_penalty: -2,
                parallel_octaves_penalty: -2,
                voice_crossing_penalty: -10,
                stepwise_bonus: 3,
                common_tone_bonus: 2,
                leap_penalty_per_semitone: 0,
                max_leap_semitones: 127,
                all_parallel_motion_penalty: -5,
                spacing_violation_penalty: 0,
                spread_preference: 5,
                contrary_motion_bonus: 3,
            },
            // Free: Closest available note, almost no rules.
            // Should sound like whatever the harmony engine gives,
            // with minimal voice leading intervention.
            VoiceLeadingStyle::Free => StyleRules {
                hard_reject_parallel_fifths: false,
                hard_reject_parallel_octaves: false,
                parallel_fifths_penalty: 0,
                parallel_octaves_penalty: 0,
                voice_crossing_penalty: -2,
                stepwise_bonus: 1,
                common_tone_bonus: 1,
                leap_penalty_per_semitone: 0,
                max_leap_semitones: 127,
                all_parallel_motion_penalty: 0,
                spacing_violation_penalty: 0,
                spread_preference: 0,
                contrary_motion_bonus: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_styles() {
        let styles = VoiceLeadingStyle::all();
        assert_eq!(styles.len(), 4);
    }

    #[test]
    fn test_default_is_free() {
        assert_eq!(VoiceLeadingStyle::default(), VoiceLeadingStyle::Free);
    }

    #[test]
    fn test_palestrina_hard_rejects() {
        let rules = StyleRules::for_style(VoiceLeadingStyle::Palestrina);
        assert!(rules.hard_reject_parallel_fifths);
        assert!(rules.hard_reject_parallel_octaves);
        assert_eq!(rules.max_leap_semitones, 5);
    }

    #[test]
    fn test_jazz_no_hard_rejects() {
        let rules = StyleRules::for_style(VoiceLeadingStyle::Jazz);
        assert!(!rules.hard_reject_parallel_fifths);
        assert!(!rules.hard_reject_parallel_octaves);
        assert_eq!(rules.max_leap_semitones, 127);
    }

    #[test]
    fn test_display() {
        let style = VoiceLeadingStyle::BachChorale;
        assert_eq!(format!("{}", style), "Bach Chorale (Baroque four-part)");
    }

    #[test]
    fn test_all_styles_construct_rules() {
        for style in VoiceLeadingStyle::all() {
            let _rules = StyleRules::for_style(*style);
        }
    }
}
