//! Metronome with accent patterns, subdivision clicks, volume control,
//! selectable sound sets, and count-in support.

use wmidi::{Channel, MidiMessage, Note, Velocity};

use super::beat_clock::SubdivisionCrossing;
use super::config::{AccentLevel, HumanizeConfig, MetronomeSubdivision};

/// Base velocities before volume scaling.
const VEL_ACCENT: u8 = 127;
const VEL_NORMAL: u8 = 100;
const VEL_GHOST: u8 = 60;
const VEL_EIGHTH_SUB: u8 = 70;
const VEL_SIXTEENTH_SUB: u8 = 50;

/// Generates MIDI metronome clicks on channel 10 (percussion).
///
/// Supports configurable accent patterns, subdivision clicks (8th/16th),
/// volume control, multiple sound sets, and count-in bars.
pub struct Metronome {
    pub enabled: bool,
    /// Tracks how many count-in beats have been emitted.
    count_in_beats_remaining: u32,
    /// Whether count-in is currently active.
    count_in_active: bool,
}

impl Metronome {
    pub fn new() -> Self {
        Self {
            enabled: false,
            count_in_beats_remaining: 0,
            count_in_active: false,
        }
    }

    /// Start a count-in sequence. Returns the total number of count-in beats.
    /// The caller should play these beats before starting the router.
    pub fn start_count_in(&mut self, config: &HumanizeConfig) -> u32 {
        let total = config.metronome_count_in_bars as u32 * config.beats_per_bar as u32;
        self.count_in_beats_remaining = total;
        self.count_in_active = total > 0;
        total
    }

    /// Returns true if count-in is still in progress.
    pub fn is_counting_in(&self) -> bool {
        self.count_in_active
    }

    /// Consume one count-in beat. Returns false when count-in is complete.
    pub fn tick_count_in(&mut self) -> bool {
        if self.count_in_beats_remaining > 0 {
            self.count_in_beats_remaining -= 1;
        }
        if self.count_in_beats_remaining == 0 {
            self.count_in_active = false;
        }
        self.count_in_active
    }

    /// Generate a click for a subdivision crossing, using the full config.
    ///
    /// Returns `Some(midi_bytes)` if a click should be played, `None` if
    /// the crossing should be silent (muted beat, or subdivision not enabled).
    pub fn generate_click_for_crossing(
        &self,
        crossing: &SubdivisionCrossing,
        config: &HumanizeConfig,
    ) -> Option<Vec<u8>> {
        if !self.enabled {
            return None;
        }

        let (accent_note, normal_note, sub_note) = config.metronome_sound.midi_notes();

        match crossing.sixteenth {
            0 => {
                // Beat downbeat — use accent pattern
                let accent = config
                    .metronome_accent_pattern
                    .get(crossing.beat as usize)
                    .unwrap_or(&AccentLevel::Normal);
                match accent {
                    AccentLevel::Mute => None,
                    AccentLevel::Accent => {
                        let vel = scale_velocity(VEL_ACCENT, config.metronome_volume);
                        Some(make_note_on(accent_note, vel))
                    }
                    AccentLevel::Normal => {
                        let vel = scale_velocity(VEL_NORMAL, config.metronome_volume);
                        Some(make_note_on(normal_note, vel))
                    }
                    AccentLevel::Ghost => {
                        let vel = scale_velocity(VEL_GHOST, config.metronome_volume);
                        Some(make_note_on(normal_note, vel))
                    }
                }
            }
            2 => {
                // 8th-note offbeat
                match config.metronome_subdivision {
                    MetronomeSubdivision::Eighth | MetronomeSubdivision::Sixteenth => {
                        let vel = scale_velocity(VEL_EIGHTH_SUB, config.metronome_volume);
                        Some(make_note_on(sub_note, vel))
                    }
                    MetronomeSubdivision::None => None,
                }
            }
            1 | 3 => {
                // 16th-note subdivisions (e and a)
                match config.metronome_subdivision {
                    MetronomeSubdivision::Sixteenth => {
                        let vel = scale_velocity(VEL_SIXTEENTH_SUB, config.metronome_volume);
                        Some(make_note_on(sub_note, vel))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Generate a NoteOff for a given MIDI note on Ch10.
    pub fn generate_click_off_for_note(&self, midi_note: u8) -> Vec<u8> {
        make_note_off(midi_note)
    }

    /// Generate a NoteOn click for the given beat number (legacy API).
    ///
    /// Beat 0 (downbeat) uses the accent note, others use the normal note.
    /// Uses Woodblock sound set and full velocity (no volume scaling).
    pub fn generate_click(&self, beat_number: u8) -> Vec<u8> {
        let midi_note = if beat_number == 0 { 76 } else { 77 };
        let vel = if beat_number == 0 {
            VEL_ACCENT
        } else {
            VEL_NORMAL
        };
        make_note_on(midi_note, vel)
    }

    /// Generate a NoteOff click for the given beat number (legacy API).
    pub fn generate_click_off(&self, beat_number: u8) -> Vec<u8> {
        let midi_note = if beat_number == 0 { 76 } else { 77 };
        make_note_off(midi_note)
    }
}

/// Apply volume scaling to a velocity value.
fn scale_velocity(base_vel: u8, volume: f32) -> u8 {
    let scaled = (base_vel as f32 * volume.clamp(0.0, 1.0)).round() as u8;
    scaled.clamp(1, 127)
}

/// Build a NoteOn MIDI message on Ch10 (percussion).
fn make_note_on(midi_note: u8, vel: u8) -> Vec<u8> {
    let note = Note::try_from(midi_note).unwrap();
    let velocity = Velocity::try_from(vel.clamp(1, 127)).unwrap();
    let msg = MidiMessage::NoteOn(Channel::Ch10, note, velocity);
    let mut buf = vec![0u8; msg.bytes_size()];
    msg.copy_to_slice(&mut buf).unwrap();
    buf
}

/// Build a NoteOff MIDI message on Ch10 (percussion).
fn make_note_off(midi_note: u8) -> Vec<u8> {
    let note = Note::try_from(midi_note).unwrap();
    let vel = Velocity::try_from(0u8).unwrap();
    let msg = MidiMessage::NoteOff(Channel::Ch10, note, vel);
    let mut buf = vec![0u8; msg.bytes_size()];
    msg.copy_to_slice(&mut buf).unwrap();
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::humanize::config::{AccentLevel, MetronomeSound, MetronomeSubdivision};

    fn test_config() -> HumanizeConfig {
        HumanizeConfig {
            metronome_enabled: true,
            metronome_volume: 1.0,
            metronome_sound: MetronomeSound::Woodblock,
            metronome_subdivision: MetronomeSubdivision::None,
            metronome_accent_pattern: vec![
                AccentLevel::Accent,
                AccentLevel::Normal,
                AccentLevel::Normal,
                AccentLevel::Normal,
            ],
            ..HumanizeConfig::default()
        }
    }

    #[test]
    fn accent_beat_generates_click() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let config = test_config();
        let crossing = SubdivisionCrossing {
            beat: 0,
            sixteenth: 0,
        };
        let click = met.generate_click_for_crossing(&crossing, &config);
        assert!(click.is_some());
        let bytes = click.unwrap();
        // Ch10 NoteOn = status 0x99, note 76, vel 127
        assert_eq!(bytes[0], 0x99);
        assert_eq!(bytes[1], 76);
        assert_eq!(bytes[2], 127);
    }

    #[test]
    fn normal_beat_generates_lower_velocity() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let config = test_config();
        let crossing = SubdivisionCrossing {
            beat: 1,
            sixteenth: 0,
        };
        let click = met.generate_click_for_crossing(&crossing, &config);
        assert!(click.is_some());
        let bytes = click.unwrap();
        assert_eq!(bytes[1], 77); // low woodblock
        assert_eq!(bytes[2], 100); // normal velocity
    }

    #[test]
    fn ghost_beat_generates_soft_click() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let mut config = test_config();
        config.metronome_accent_pattern[2] = AccentLevel::Ghost;
        let crossing = SubdivisionCrossing {
            beat: 2,
            sixteenth: 0,
        };
        let click = met.generate_click_for_crossing(&crossing, &config);
        assert!(click.is_some());
        let bytes = click.unwrap();
        assert_eq!(bytes[2], VEL_GHOST);
    }

    #[test]
    fn muted_beat_generates_nothing() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let mut config = test_config();
        config.metronome_accent_pattern[1] = AccentLevel::Mute;
        let crossing = SubdivisionCrossing {
            beat: 1,
            sixteenth: 0,
        };
        assert!(met
            .generate_click_for_crossing(&crossing, &config)
            .is_none());
    }

    #[test]
    fn eighth_subdivision_generates_click() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let mut config = test_config();
        config.metronome_subdivision = MetronomeSubdivision::Eighth;
        let crossing = SubdivisionCrossing {
            beat: 0,
            sixteenth: 2,
        };
        let click = met.generate_click_for_crossing(&crossing, &config);
        assert!(click.is_some());
        let bytes = click.unwrap();
        assert_eq!(bytes[1], 42); // hi-hat for subdivision
        assert_eq!(bytes[2], VEL_EIGHTH_SUB);
    }

    #[test]
    fn sixteenth_subdivision_generates_click() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let mut config = test_config();
        config.metronome_subdivision = MetronomeSubdivision::Sixteenth;
        let crossing = SubdivisionCrossing {
            beat: 0,
            sixteenth: 1,
        };
        let click = met.generate_click_for_crossing(&crossing, &config);
        assert!(click.is_some());
        let bytes = click.unwrap();
        assert_eq!(bytes[1], 42);
        assert_eq!(bytes[2], VEL_SIXTEENTH_SUB);
    }

    #[test]
    fn no_subdivision_skips_offbeat() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let config = test_config(); // subdivision = None
        let crossing = SubdivisionCrossing {
            beat: 0,
            sixteenth: 2,
        };
        assert!(met
            .generate_click_for_crossing(&crossing, &config)
            .is_none());
    }

    #[test]
    fn volume_scaling() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let mut config = test_config();
        config.metronome_volume = 0.5;
        let crossing = SubdivisionCrossing {
            beat: 0,
            sixteenth: 0,
        };
        let click = met.generate_click_for_crossing(&crossing, &config).unwrap();
        // 127 * 0.5 = 63.5 -> 64
        assert_eq!(click[2], 64);
    }

    #[test]
    fn sound_set_rimshot() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let mut config = test_config();
        config.metronome_sound = MetronomeSound::Rimshot;
        let crossing = SubdivisionCrossing {
            beat: 0,
            sixteenth: 0,
        };
        let click = met.generate_click_for_crossing(&crossing, &config).unwrap();
        assert_eq!(click[1], 37); // rimshot note
    }

    #[test]
    fn sound_set_cowbell() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let mut config = test_config();
        config.metronome_sound = MetronomeSound::Cowbell;
        let crossing = SubdivisionCrossing {
            beat: 0,
            sixteenth: 0,
        };
        let click = met.generate_click_for_crossing(&crossing, &config).unwrap();
        assert_eq!(click[1], 56); // cowbell note
    }

    #[test]
    fn count_in_lifecycle() {
        let mut met = Metronome::new();
        met.enabled = true;
        let mut config = test_config();
        config.metronome_count_in_bars = 2;
        config.beats_per_bar = 4;

        let total = met.start_count_in(&config);
        assert_eq!(total, 8); // 2 bars * 4 beats
        assert!(met.is_counting_in());

        for i in 0..7 {
            assert!(
                met.tick_count_in(),
                "beat {} should still be counting in",
                i
            );
        }
        assert!(!met.tick_count_in()); // 8th beat finishes count-in
        assert!(!met.is_counting_in());
    }

    #[test]
    fn disabled_metronome_generates_nothing() {
        let met = Metronome::new(); // enabled = false
        let config = test_config();
        let crossing = SubdivisionCrossing {
            beat: 0,
            sixteenth: 0,
        };
        assert!(met
            .generate_click_for_crossing(&crossing, &config)
            .is_none());
    }

    #[test]
    fn legacy_generate_click_still_works() {
        let met = Metronome {
            enabled: true,
            count_in_beats_remaining: 0,
            count_in_active: false,
        };
        let click = met.generate_click(0);
        assert_eq!(click[0], 0x99); // Ch10 NoteOn
        assert_eq!(click[1], 76); // high woodblock
        assert_eq!(click[2], 127); // accent velocity

        let click = met.generate_click(2);
        assert_eq!(click[1], 77); // low woodblock
        assert_eq!(click[2], 100); // normal velocity
    }
}
