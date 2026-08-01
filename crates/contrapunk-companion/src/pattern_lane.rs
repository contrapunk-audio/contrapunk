//! Declarative transport pattern for one stable pitched role.
//!
//! A PatternLane owns one monophonic role (for example low support or
//! counterline). Input NoteOns arm a bounded transport window; the role's
//! configured scale-degree events then run independently of the input rhythm.
//! Presets provide the data. The shared lane contains no preset-specific music.

use contrapunk_harmony::{Key, Scale, ScaleMode};
use wmidi::Note;

use super::lane::{InputEvent, InputFilter, Lane, LaneOutput, LanePhase};
use super::voice_output::VoiceOutputTarget;
use super::world::WorldState;
use super::DispatchOp;

const MAX_EVENTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PatternPitchAnchor {
    Key,
    PhraseStart,
    LatestInput,
}

impl PatternPitchAnchor {
    fn as_str(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::PhraseStart => "phrase_start",
            Self::LatestInput => "latest_input",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "key" => Ok(Self::Key),
            "phrase_start" => Ok(Self::PhraseStart),
            "latest_input" => Ok(Self::LatestInput),
            _ => Err("pattern pitch_anchor must be key, phrase_start, or latest_input".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatternEvent {
    pub beat: f64,
    pub degree: u8,
    pub octave: i8,
    pub duration_beats: f64,
    pub velocity: u8,
}

#[derive(Clone, Copy, Debug)]
struct ActiveNote {
    note: u8,
    channel: u8,
    off_at: f64,
}

pub struct PatternLane {
    name: &'static str,
    type_id: &'static str,
    enabled: bool,
    cycle_beats: f64,
    tail_beats: f64,
    events: Vec<PatternEvent>,
    target: VoiceOutputTarget,
    pitch_anchor: PatternPitchAnchor,
    only_when_input_idle: bool,
    anchor: Option<f64>,
    phrase_pitch_class: Option<u8>,
    cycle_start: f64,
    next_event: usize,
    active_until: f64,
    channel: u8,
    active: Option<ActiveNote>,
    phrase_id: Option<u64>,
    last_tick_beat: Option<f64>,
}

impl PatternLane {
    pub fn new(name: &'static str, type_id: &'static str) -> Self {
        Self {
            name,
            type_id,
            enabled: false,
            cycle_beats: 4.0,
            tail_beats: 4.0,
            events: Vec::new(),
            target: VoiceOutputTarget::Synth,
            pitch_anchor: PatternPitchAnchor::Key,
            only_when_input_idle: false,
            anchor: None,
            phrase_pitch_class: None,
            cycle_start: 0.0,
            next_event: 0,
            active_until: 0.0,
            channel: 0,
            active: None,
            phrase_id: None,
            last_tick_beat: None,
        }
    }

    fn clear_schedule(&mut self) {
        self.anchor = None;
        self.phrase_pitch_class = None;
        self.cycle_start = 0.0;
        self.next_event = 0;
        self.active_until = 0.0;
        self.phrase_id = None;
        self.last_tick_beat = None;
    }

    fn release_active(&mut self, ops: &mut Vec<DispatchOp>) {
        if let Some(active) = self.active.take() {
            ops.push(DispatchOp::NoteOff {
                target: self.target,
                note: active.note,
                channel: active.channel,
            });
        }
    }

    fn advance_event(&mut self) {
        self.next_event += 1;
        if self.next_event >= self.events.len() {
            self.next_event = 0;
            self.cycle_start += self.cycle_beats;
        }
    }

    fn resolve_note(&self, world: &WorldState, event: PatternEvent) -> Option<u8> {
        let (key, mode) = world
            .engine_snapshot
            .lock()
            .map(|engine| (engine.key(), engine.scale_mode()))
            .unwrap_or((Key::C, ScaleMode::Ionian));
        let offsets = mode.intervals();
        let tonic = key.semitones_from_c() as usize;
        let degree = event.degree as usize;
        let midi = match self.pitch_anchor {
            PatternPitchAnchor::Key => {
                60 + tonic as i16 + *offsets.get(degree)? as i16 + 12 * event.octave as i16
            }
            PatternPitchAnchor::PhraseStart | PatternPitchAnchor::LatestInput => {
                let phrase_pitch_class = self.phrase_pitch_class.unwrap_or(tonic as u8);
                let scale = Scale::new(tonic as u8, mode);
                let anchor = scale.snap_to_scale(Note::try_from(60u8 + phrase_pitch_class).ok()?);
                let anchor_degree = scale.degree_of(anchor)?;
                let target_degree = anchor_degree + degree;
                60 + tonic as i16
                    + offsets[target_degree % offsets.len()] as i16
                    + 12 * (event.octave as i16 + (target_degree / offsets.len()) as i16)
            }
        };
        u8::try_from(midi).ok().filter(|midi| *midi <= 127)
    }

    fn parse_events(
        value: &serde_json::Value,
        cycle_beats: f64,
    ) -> Result<Vec<PatternEvent>, String> {
        let Some(items) = value.as_array() else {
            return Err("pattern events must be an array".into());
        };
        if items.len() > MAX_EVENTS {
            return Err(format!("pattern supports at most {MAX_EVENTS} events"));
        }
        let mut events = Vec::with_capacity(items.len());
        for item in items {
            let beat = item
                .get("beat")
                .and_then(|value| value.as_f64())
                .ok_or_else(|| "pattern event beat must be a number".to_string())?;
            let degree = item
                .get("degree")
                .and_then(|value| value.as_u64())
                .ok_or_else(|| "pattern event degree must be an integer".to_string())?;
            let octave = item
                .get("octave")
                .and_then(|value| value.as_i64())
                .ok_or_else(|| "pattern event octave must be an integer".to_string())?;
            let duration_beats = item
                .get("duration_beats")
                .and_then(|value| value.as_f64())
                .ok_or_else(|| "pattern event duration_beats must be a number".to_string())?;
            let velocity = item
                .get("velocity")
                .and_then(|value| value.as_u64())
                .ok_or_else(|| "pattern event velocity must be an integer".to_string())?;
            if !beat.is_finite() || beat < 0.0 || beat >= cycle_beats {
                return Err("pattern event beat must fall inside the cycle".into());
            }
            if degree > 6 || !(-4..=4).contains(&octave) {
                return Err("pattern event degree/octave is out of range".into());
            }
            if !duration_beats.is_finite() || !(0.03125..=32.0).contains(&duration_beats) {
                return Err("pattern event duration must be between 0.03125 and 32 beats".into());
            }
            if !(1..=127).contains(&velocity) {
                return Err("pattern event velocity must be between 1 and 127".into());
            }
            events.push(PatternEvent {
                beat,
                degree: degree as u8,
                octave: octave as i8,
                duration_beats,
                velocity: velocity as u8,
            });
        }
        events.sort_by(|left, right| left.beat.total_cmp(&right.beat));
        Ok(events)
    }
}

impl Lane for PatternLane {
    fn name(&self) -> &str {
        self.name
    }

    fn type_id(&self) -> &'static str {
        self.type_id
    }

    fn phase(&self) -> LanePhase {
        LanePhase::Decide
    }

    fn input_filter(&self) -> InputFilter {
        InputFilter::All
    }

    fn reset_runtime(&mut self) {
        self.active = None;
        self.clear_schedule();
    }

    fn on_input(&mut self, ev: InputEvent, world: &WorldState) -> LaneOutput {
        let (note, channel) = match ev {
            InputEvent::NoteOn { note, channel, .. } => (note, channel),
            InputEvent::NoteOff { .. } | InputEvent::Cc { .. } => {
                return LaneOutput::default();
            }
        };
        if !self.enabled {
            return LaneOutput::default();
        }

        let mut ops = Vec::new();
        if self.only_when_input_idle {
            self.release_active(&mut ops);
        }
        if self.events.is_empty() || !world.transport.is_running() {
            return LaneOutput {
                ops,
                ..Default::default()
            };
        }

        let now = world.transport.total_beats();
        let phrase = world.phrase_snapshot();
        let phrase_relative = self.pitch_anchor != PatternPitchAnchor::Key;
        let new_phrase = phrase_relative && phrase.id != self.phrase_id;
        if new_phrase || self.anchor.is_none() || now >= self.active_until {
            self.release_active(&mut ops);
            self.anchor = Some(now);
            self.phrase_pitch_class = Some(
                match self.pitch_anchor {
                    PatternPitchAnchor::PhraseStart => phrase.opening_note.unwrap_or(note),
                    PatternPitchAnchor::LatestInput => phrase.latest_note.unwrap_or(note),
                    PatternPitchAnchor::Key => note,
                } % 12,
            );
            self.cycle_start = now;
            self.next_event = 0;
        } else if self.pitch_anchor == PatternPitchAnchor::LatestInput {
            self.phrase_pitch_class = Some(phrase.latest_note.unwrap_or(note) % 12);
        }
        self.phrase_id = phrase.id;
        self.active_until = now + self.tail_beats;
        self.channel = phrase.latest_channel.unwrap_or(channel);
        LaneOutput {
            ops,
            ..Default::default()
        }
    }

    fn tick(&mut self, world: &WorldState) -> LaneOutput {
        let now = world.transport.total_beats();
        let mut ops = Vec::new();

        let rewound = self
            .last_tick_beat
            .map(|previous| now + f64::EPSILON < previous)
            .unwrap_or(false);
        self.last_tick_beat = Some(now);
        if !self.enabled || !world.transport.is_running() || rewound {
            self.release_active(&mut ops);
            self.clear_schedule();
            return LaneOutput {
                ops,
                ..Default::default()
            };
        }

        if self.active.map(|note| note.off_at <= now).unwrap_or(false) {
            self.release_active(&mut ops);
        }
        if self.anchor.is_some() && now >= self.active_until {
            self.release_active(&mut ops);
            self.clear_schedule();
            return LaneOutput {
                ops,
                ..Default::default()
            };
        }

        while self.anchor.is_some() && !self.events.is_empty() {
            let event = self.events[self.next_event];
            let fire_at = self.cycle_start + event.beat;
            if fire_at >= self.active_until || fire_at > now {
                break;
            }

            let off_at = fire_at + event.duration_beats;
            if off_at > now && (!self.only_when_input_idle || world.phrase_snapshot().input_idle) {
                self.release_active(&mut ops);
                if let Some(note) = self.resolve_note(world, event) {
                    ops.push(DispatchOp::NoteOn {
                        target: self.target,
                        note,
                        velocity: event.velocity,
                        channel: self.channel,
                    });
                    self.active = Some(ActiveNote {
                        note,
                        channel: self.channel,
                        off_at,
                    });
                }
            }
            self.advance_event();
        }

        LaneOutput {
            ops,
            ..Default::default()
        }
    }

    fn serialize_state(&self) -> serde_json::Value {
        let events: Vec<_> = self
            .events
            .iter()
            .map(|event| {
                serde_json::json!({
                    "beat": event.beat,
                    "degree": event.degree,
                    "octave": event.octave,
                    "duration_beats": event.duration_beats,
                    "velocity": event.velocity,
                })
            })
            .collect();
        serde_json::json!({
            "enabled": self.enabled,
            "cycle_beats": self.cycle_beats,
            "tail_beats": self.tail_beats,
            "pitch_anchor": self.pitch_anchor.as_str(),
            "only_when_input_idle": self.only_when_input_idle,
            "events": events,
        })
    }

    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
        let cycle_beats = state
            .get("cycle_beats")
            .and_then(|value| value.as_f64())
            .unwrap_or(self.cycle_beats);
        if !cycle_beats.is_finite() || !(0.25..=32.0).contains(&cycle_beats) {
            return Err("pattern cycle_beats must be between 0.25 and 32".into());
        }
        let tail_beats = state
            .get("tail_beats")
            .and_then(|value| value.as_f64())
            .unwrap_or(self.tail_beats);
        if !tail_beats.is_finite() || !(0.25..=32.0).contains(&tail_beats) {
            return Err("pattern tail_beats must be between 0.25 and 32".into());
        }
        let events = state
            .get("events")
            .map(|value| Self::parse_events(value, cycle_beats))
            .transpose()?;
        let enabled = state.get("enabled").and_then(|value| value.as_bool());
        let pitch_anchor = state
            .get("pitch_anchor")
            .and_then(|value| value.as_str())
            .map(PatternPitchAnchor::parse)
            .transpose()?
            .unwrap_or(self.pitch_anchor);
        let only_when_input_idle = state
            .get("only_when_input_idle")
            .and_then(|value| value.as_bool())
            .unwrap_or(self.only_when_input_idle);
        let schedule_changed = state.get("cycle_beats").is_some()
            || state.get("tail_beats").is_some()
            || state.get("pitch_anchor").is_some()
            || state.get("only_when_input_idle").is_some()
            || events.is_some()
            || enabled.map(|value| value != self.enabled).unwrap_or(false);

        self.cycle_beats = cycle_beats;
        self.tail_beats = tail_beats;
        self.pitch_anchor = pitch_anchor;
        self.only_when_input_idle = only_when_input_idle;
        if let Some(events) = events {
            self.events = events;
        }
        if let Some(enabled) = enabled {
            self.enabled = enabled;
        }
        if schedule_changed {
            // Configuration cannot emit cleanup itself. Preserve a sounding
            // note so the next tick can release it naturally (or immediately
            // when disabled), but never continue indexing the old schedule.
            self.clear_schedule();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use contrapunk_harmony::{HarmonyEngine, HarmonyMode, Key};
    use contrapunk_transport::Transport;

    use super::*;

    fn fixture(type_id: &'static str) -> (PatternLane, Arc<WorldState>, Arc<Transport>) {
        let transport = Transport::new(48_000);
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::PassThrough,
        )));
        let world = WorldState::new(Arc::clone(&transport), engine);
        (PatternLane::new("Pattern", type_id), world, transport)
    }

    fn advance_to_beat(transport: &Transport, beat: f64) {
        let frames_per_beat = transport.sample_rate() as f64 * 60.0 / transport.bpm();
        transport.reset();
        transport.play();
        let frames = (beat * frames_per_beat) as u32;
        if frames > 0 {
            let _ = transport.advance(frames);
        }
    }

    fn configure(lane: &mut PatternLane, beat: f64, degree: u8, octave: i8, duration: f64) {
        lane.deserialize_state(serde_json::json!({
            "enabled": true,
            "cycle_beats": 4.0,
            "tail_beats": 4.0,
            "events": [{
                "beat": beat,
                "degree": degree,
                "octave": octave,
                "duration_beats": duration,
                "velocity": 80
            }]
        }))
        .unwrap();
    }

    #[test]
    fn two_roles_have_independent_timing_and_source_independent_bass() {
        let (mut low, world, transport) = fixture("pattern_low");
        let mut counter = PatternLane::new("Counter", "pattern_counter");
        configure(&mut low, 0.0, 0, -2, 1.25);
        configure(&mut counter, 1.0, 4, 0, 0.75);

        advance_to_beat(&transport, 0.0);
        let opening = InputEvent::NoteOn {
            note: 67,
            velocity: 110,
            channel: 2,
        };
        world.observe_phrase_input(opening);
        for lane in [&mut low, &mut counter] {
            lane.on_input(opening, &world);
        }

        advance_to_beat(&transport, 0.01);
        assert_eq!(
            low.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 36,
                velocity: 80,
                channel: 2,
            }]
        );
        assert!(counter.tick(&world).ops.is_empty());

        advance_to_beat(&transport, 1.01);
        assert_eq!(
            counter.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 67,
                velocity: 80,
                channel: 2,
            }]
        );
        assert!(low.active.is_some());
        assert!(counter.active.is_some());
    }

    #[test]
    fn pixel_trio_cycle_tracks_latest_input_and_has_a_clean_tail() {
        let (mut low, world, transport) = fixture("pattern_low");
        let mut counter = PatternLane::new("Counter", "pattern_counter");
        low.deserialize_state(serde_json::json!({
            "enabled": true,
            "cycle_beats": 4,
            "tail_beats": 4,
            "pitch_anchor": "phrase_start",
            "events": [
                {"beat": 0, "degree": 0, "octave": -2, "duration_beats": 0.5, "velocity": 72},
                {"beat": 1, "degree": 4, "octave": -2, "duration_beats": 0.375, "velocity": 64},
                {"beat": 2, "degree": 2, "octave": -2, "duration_beats": 0.5, "velocity": 68},
                {"beat": 3, "degree": 4, "octave": -2, "duration_beats": 0.375, "velocity": 62}
            ]
        }))
        .unwrap();
        counter
            .deserialize_state(serde_json::json!({
                "enabled": true,
                "cycle_beats": 4,
                "tail_beats": 4,
                "pitch_anchor": "latest_input",
                "only_when_input_idle": true,
                "events": [
                    {"beat": 0.5, "degree": 4, "octave": 0, "duration_beats": 1, "velocity": 60},
                    {"beat": 1.5, "degree": 2, "octave": 0, "duration_beats": 1, "velocity": 56},
                    {"beat": 2.5, "degree": 5, "octave": 0, "duration_beats": 1, "velocity": 58},
                    {"beat": 3.5, "degree": 4, "octave": 0, "duration_beats": 1, "velocity": 54}
                ]
            }))
            .unwrap();

        advance_to_beat(&transport, 0.0);
        let opening = InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(opening);
        for lane in [&mut low, &mut counter] {
            lane.on_input(opening, &world);
        }

        let mut note_ons = Vec::new();
        for beat in [0.01, 0.51] {
            advance_to_beat(&transport, beat);
            for (lane_id, lane) in [("pattern_low", &mut low), ("pattern_counter", &mut counter)] {
                for op in lane.tick(&world).ops {
                    if let DispatchOp::NoteOn { note, velocity, .. } = op {
                        note_ons.push((lane_id, note, velocity));
                    }
                }
            }
            if beat == 0.01 {
                let release = InputEvent::NoteOff {
                    note: 60,
                    channel: 0,
                };
                world.observe_phrase_input(release);
                for lane in [&mut low, &mut counter] {
                    lane.on_input(release, &world);
                }
            }
        }

        advance_to_beat(&transport, 0.75);
        let next = InputEvent::NoteOn {
            note: 62,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(next);
        for lane in [&mut low, &mut counter] {
            lane.on_input(next, &world);
        }
        let release = InputEvent::NoteOff {
            note: 62,
            channel: 0,
        };
        world.observe_phrase_input(release);
        for lane in [&mut low, &mut counter] {
            lane.on_input(release, &world);
        }
        for beat in [1.01, 1.51, 2.01, 2.51, 3.01, 3.51] {
            advance_to_beat(&transport, beat);
            for (lane_id, lane) in [("pattern_low", &mut low), ("pattern_counter", &mut counter)] {
                let ops = lane.tick(&world).ops;
                if lane_id == "pattern_counter" && beat == 2.51 {
                    assert_eq!(
                        ops,
                        vec![
                            DispatchOp::NoteOff {
                                target: VoiceOutputTarget::Synth,
                                note: 65,
                                channel: 0,
                            },
                            DispatchOp::NoteOn {
                                target: VoiceOutputTarget::Synth,
                                note: 71,
                                velocity: 58,
                                channel: 0,
                            },
                        ]
                    );
                }
                for op in ops {
                    if let DispatchOp::NoteOn { note, velocity, .. } = op {
                        note_ons.push((lane_id, note, velocity));
                    }
                }
            }
        }

        assert_eq!(
            note_ons,
            vec![
                ("pattern_low", 36, 72),
                ("pattern_counter", 67, 60),
                ("pattern_low", 43, 64),
                ("pattern_counter", 65, 56),
                ("pattern_low", 40, 68),
                ("pattern_counter", 71, 58),
                ("pattern_low", 43, 62),
                ("pattern_counter", 69, 54),
            ]
        );

        advance_to_beat(&transport, 4.76);
        for lane in [&mut low, &mut counter] {
            lane.tick(&world);
            assert!(lane.active.is_none());
            assert!(lane.anchor.is_none());
        }
    }

    #[test]
    fn tail_stops_and_releases_without_resurrection() {
        let (mut lane, world, transport) = fixture("pattern_low");
        configure(&mut lane, 0.0, 0, -2, 5.0);
        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.01);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [DispatchOp::NoteOn { .. }]
        ));
        advance_to_beat(&transport, 4.01);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [DispatchOp::NoteOff { .. }]
        ));
        assert!(lane.anchor.is_none());
        assert!(lane.active.is_none());
        lane.reset_runtime();
        advance_to_beat(&transport, 8.0);
        assert!(lane.tick(&world).ops.is_empty());
    }

    #[test]
    fn configured_phrase_gap_reanchors_before_the_old_pattern_tail() {
        let (mut lane, world, transport) = fixture("pattern_low");
        world.set_phrase_gap_beats(0.5).unwrap();
        lane.deserialize_state(serde_json::json!({
            "enabled": true,
            "cycle_beats": 4,
            "tail_beats": 4,
            "pitch_anchor": "phrase_start",
            "events": [
                {"beat": 0, "degree": 0, "octave": 0, "duration_beats": 5, "velocity": 72}
            ]
        }))
        .unwrap();
        advance_to_beat(&transport, 0.0);
        let opening = InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(opening);
        lane.on_input(opening, &world);
        advance_to_beat(&transport, 0.01);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 60,
                velocity: 72,
                channel: 0,
            }]
        );
        world.observe_phrase_input(InputEvent::NoteOff {
            note: 60,
            channel: 0,
        });

        advance_to_beat(&transport, 1.0);
        let next_phrase = InputEvent::NoteOn {
            note: 67,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(next_phrase);
        assert_eq!(
            lane.on_input(next_phrase, &world).ops,
            vec![DispatchOp::NoteOff {
                target: VoiceOutputTarget::Synth,
                note: 60,
                channel: 0,
            }]
        );
        advance_to_beat(&transport, 1.01);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 67,
                velocity: 72,
                channel: 0,
            }]
        );
    }

    #[test]
    fn transport_stop_releases_active_role_and_clears_schedule() {
        let (mut lane, world, transport) = fixture("pattern_low");
        configure(&mut lane, 0.0, 0, -2, 2.0);
        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.01);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [DispatchOp::NoteOn { .. }]
        ));

        transport.stop();
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [DispatchOp::NoteOff { .. }]
        ));
        assert!(lane.active.is_none());
        assert!(lane.anchor.is_none());
    }

    #[test]
    fn reconfigure_resets_old_cursor_without_dropping_active_release() {
        let (mut lane, world, transport) = fixture("pattern_low");
        lane.deserialize_state(serde_json::json!({
            "enabled": true,
            "cycle_beats": 4,
            "tail_beats": 4,
            "events": [
                {"beat": 0, "degree": 0, "octave": -2, "duration_beats": 2, "velocity": 80},
                {"beat": 1, "degree": 4, "octave": -2, "duration_beats": 1, "velocity": 70}
            ]
        }))
        .unwrap();
        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.01);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [DispatchOp::NoteOn { .. }]
        ));

        lane.deserialize_state(serde_json::json!({
            "events": [
                {"beat": 0, "degree": 2, "octave": -1, "duration_beats": 1, "velocity": 60}
            ]
        }))
        .unwrap();
        assert!(lane.anchor.is_none());
        assert_eq!(lane.next_event, 0);
        assert!(lane.active.is_some());

        advance_to_beat(&transport, 2.01);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [DispatchOp::NoteOff { .. }]
        ));
        assert!(lane.active.is_none());
    }

    #[test]
    fn phrase_start_anchor_moves_the_pattern_diatonically_from_the_opening_pitch() {
        let (mut lane, world, transport) = fixture("pattern_low");
        lane.deserialize_state(serde_json::json!({
            "enabled": true,
            "cycle_beats": 4,
            "tail_beats": 4,
            "pitch_anchor": "phrase_start",
            "events": [
                {"beat": 0, "degree": 2, "octave": -2, "duration_beats": 1, "velocity": 72}
            ]
        }))
        .unwrap();

        advance_to_beat(&transport, 0.0);
        let opening = InputEvent::NoteOn {
            note: 64,
            velocity: 100,
            channel: 1,
        };
        world.observe_phrase_input(opening);
        lane.on_input(opening, &world);
        advance_to_beat(&transport, 0.01);

        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 43,
                velocity: 72,
                channel: 1,
            }]
        );
        let state = lane.serialize_state();
        assert_eq!(state["pitch_anchor"], "phrase_start");
        assert_eq!(state["only_when_input_idle"], false);
    }

    #[test]
    fn chromatic_anchor_snaps_to_the_scale_before_transposing() {
        let (mut lane, world, transport) = fixture("pattern_counter");
        lane.deserialize_state(serde_json::json!({
            "enabled": true,
            "cycle_beats": 4,
            "tail_beats": 4,
            "pitch_anchor": "latest_input",
            "events": [
                {"beat": 0, "degree": 4, "octave": 0, "duration_beats": 1, "velocity": 60}
            ]
        }))
        .unwrap();

        advance_to_beat(&transport, 0.0);
        let opening = InputEvent::NoteOn {
            note: 61,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(opening);
        lane.on_input(opening, &world);
        advance_to_beat(&transport, 0.01);

        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 67,
                velocity: 60,
                channel: 0,
            }]
        );
    }

    #[test]
    fn gap_only_pattern_skips_held_input_and_yields_to_a_new_attack() {
        let (mut lane, world, transport) = fixture("pattern_counter");
        lane.deserialize_state(serde_json::json!({
            "enabled": true,
            "cycle_beats": 4,
            "tail_beats": 4,
            "only_when_input_idle": true,
            "events": [
                {"beat": 0, "degree": 0, "octave": -1, "duration_beats": 0.5, "velocity": 70},
                {"beat": 1, "degree": 4, "octave": -1, "duration_beats": 1, "velocity": 68}
            ]
        }))
        .unwrap();

        advance_to_beat(&transport, 0.0);
        let opening = InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(opening);
        lane.on_input(opening, &world);
        advance_to_beat(&transport, 0.01);
        assert!(lane.tick(&world).ops.is_empty());

        let release = InputEvent::NoteOff {
            note: 60,
            channel: 0,
        };
        world.observe_phrase_input(release);
        lane.on_input(release, &world);
        advance_to_beat(&transport, 1.01);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 55,
                velocity: 68,
                channel: 0,
            }]
        );

        let attack = InputEvent::NoteOn {
            note: 64,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(attack);
        let yielded = lane.on_input(attack, &world);
        assert_eq!(
            yielded.ops,
            vec![DispatchOp::NoteOff {
                target: VoiceOutputTarget::Synth,
                note: 55,
                channel: 0,
            }]
        );
    }

    #[test]
    fn repeated_input_note_keeps_the_gap_closed_until_every_release() {
        let (_lane, world, transport) = fixture("pattern_counter");
        advance_to_beat(&transport, 0.0);
        let note_on = InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(note_on);
        world.observe_phrase_input(note_on);
        world.observe_phrase_input(InputEvent::NoteOff {
            note: 60,
            channel: 1,
        });
        assert!(!world.phrase_snapshot().input_idle);
        world.observe_phrase_input(InputEvent::NoteOff {
            note: 60,
            channel: 0,
        });
        assert!(!world.phrase_snapshot().input_idle);
        world.observe_phrase_input(InputEvent::NoteOff {
            note: 60,
            channel: 0,
        });
        assert!(world.phrase_snapshot().input_idle);
    }

    #[test]
    fn rejects_invalid_event_data() {
        let (mut lane, _world, _transport) = fixture("pattern_low");
        assert!(lane
            .deserialize_state(serde_json::json!({
                "cycle_beats": 4.0,
                "events": [{"beat": 4.0, "degree": 0, "octave": -2, "duration_beats": 1, "velocity": 80}]
            }))
            .is_err());
        assert!(lane
            .deserialize_state(serde_json::json!({ "pitch_anchor": "random" }))
            .is_err());
    }
}
