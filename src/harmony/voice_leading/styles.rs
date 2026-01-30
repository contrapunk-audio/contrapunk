//! Voice leading style presets with hard vs soft constraint distinction.

/// Voice leading style presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
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
}

impl StyleRules {
    /// Returns the style rules for a given voice leading style.
    pub fn for_style(style: VoiceLeadingStyle) -> StyleRules {
        match style {
            VoiceLeadingStyle::Palestrina => StyleRules {
                hard_reject_parallel_fifths: true,
                hard_reject_parallel_octaves: true,
                parallel_fifths_penalty: -50,
                parallel_octaves_penalty: -50,
                voice_crossing_penalty: -80,
                stepwise_bonus: 15,
                common_tone_bonus: 12,
                leap_penalty_per_semitone: -3,
                max_leap_semitones: 7,
                all_parallel_motion_penalty: -50,
                spacing_violation_penalty: -100,
            },
            VoiceLeadingStyle::BachChorale => StyleRules {
                hard_reject_parallel_fifths: true,
                hard_reject_parallel_octaves: true,
                parallel_fifths_penalty: -40,
                parallel_octaves_penalty: -40,
                voice_crossing_penalty: -60,
                stepwise_bonus: 10,
                common_tone_bonus: 10,
                leap_penalty_per_semitone: -2,
                max_leap_semitones: 12,
                all_parallel_motion_penalty: -30,
                spacing_violation_penalty: -50,
            },
            VoiceLeadingStyle::Jazz => StyleRules {
                hard_reject_parallel_fifths: false,
                hard_reject_parallel_octaves: false,
                parallel_fifths_penalty: -5,
                parallel_octaves_penalty: -5,
                voice_crossing_penalty: -20,
                stepwise_bonus: 4,
                common_tone_bonus: 3,
                leap_penalty_per_semitone: -1,
                max_leap_semitones: 127,
                all_parallel_motion_penalty: -10,
                spacing_violation_penalty: -10,
            },
            VoiceLeadingStyle::Free => StyleRules {
                hard_reject_parallel_fifths: false,
                hard_reject_parallel_octaves: false,
                parallel_fifths_penalty: -2,
                parallel_octaves_penalty: -2,
                voice_crossing_penalty: -5,
                stepwise_bonus: 3,
                common_tone_bonus: 2,
                leap_penalty_per_semitone: 0,
                max_leap_semitones: 127,
                all_parallel_motion_penalty: -5,
                spacing_violation_penalty: -5,
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
        assert_eq!(rules.max_leap_semitones, 7);
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
