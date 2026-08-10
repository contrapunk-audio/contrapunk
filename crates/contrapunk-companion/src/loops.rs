//! One volatile, input-only MIDI phrase loop.
//!
//! The core owns musical timing and capture state but no transport, router, or
//! UI. Adapters pass absolute beat positions into `press`, `capture`, and
//! `tick`, then route replay events through an isolated arrangement runtime.

use serde::Serialize;

pub const MICROBEATS_PER_BEAT: u64 = 1_000_000;
// ponytail: fixed safety ceilings; make them configurable only if real sessions
// hit 256 beats or 16,384 captured events.
pub const MAX_LOOP_BEATS: u64 = 256;
pub const MAX_LOOP_EVENTS: usize = 16_384;

const MIDI_NOTES: usize = 128;
const MIDI_CHANNELS: usize = 16;
const MIDI_OWNERS: usize = MIDI_NOTES * MIDI_CHANNELS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputOrigin {
    Live,
    Loop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginMidiEvent {
    pub origin: InputOrigin,
    pub event: LoopMidiEvent,
    /// Absolute replay time in the looper's fixed microbeat domain.
    /// Live capture events are untimed here because `capture` receives time.
    pub scheduled_beat_us: Option<u64>,
}

impl OriginMidiEvent {
    pub fn scheduled_beat(self) -> Option<f64> {
        self.scheduled_beat_us
            .map(|beat| beat as f64 / MICROBEATS_PER_BEAT as f64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopMidiEvent {
    NoteOn { note: u8, velocity: u8, channel: u8 },
    NoteOff { note: u8, velocity: u8, channel: u8 },
    Cc64 { value: u8, channel: u8 },
}

impl LoopMidiEvent {
    fn valid(self) -> bool {
        match self {
            Self::NoteOn {
                note,
                velocity,
                channel,
            }
            | Self::NoteOff {
                note,
                velocity,
                channel,
            } => {
                (note as usize) < MIDI_NOTES
                    && velocity <= 127
                    && (channel as usize) < MIDI_CHANNELS
            }
            Self::Cc64 { value, channel } => value <= 127 && (channel as usize) < MIDI_CHANNELS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LoopEvent {
    /// Offset from loop start. Ordinary events are in `[0, length)`.
    pub beat_offset_us: u64,
    pub event: LoopMidiEvent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LoopBuffer {
    pub length_beats_us: u64,
    pub recorded_beats_per_bar_us: u64,
    pub events: Vec<LoopEvent>,
    /// Synthetic closure events at `length`. Playback emits these before
    /// offset-zero events whenever a cycle wraps.
    pub boundary_events: Vec<LoopMidiEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopState {
    Empty,
    Armed {
        start_beat_us: u64,
        beats_per_bar_us: u64,
    },
    Recording {
        start_beat_us: u64,
        beats_per_bar_us: u64,
        close_beat_us: Option<u64>,
    },
    Playing {
        origin_beat_us: u64,
    },
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopStatusState {
    Empty,
    Armed,
    Recording,
    Closing,
    Playing,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct LoopStatus {
    pub state: LoopStatusState,
    pub close_pending: bool,
    pub has_loop: bool,
    pub current_beats: f64,
    pub recorded_beats: f64,
    pub recorded_bars: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoopPressOutcome {
    /// Start a stopped transport. Resume does not imply a reset.
    pub start_transport: bool,
    /// Reset to beat zero before starting a new recording count-in.
    pub reset_transport: bool,
}

pub struct LooperLane {
    state: LoopState,
    buffer: Option<LoopBuffer>,
    capture_events: Vec<LoopEvent>,
    pressed: [u16; MIDI_OWNERS],
    sustain_down: [bool; MIDI_CHANNELS],
    captured_note_on: bool,
    playback_cursor_us: Option<u64>,
    last_transport_beat_us: Option<u64>,
    accepted_discontinuity_revision: Option<u64>,
    cleanup_requested: bool,
}

impl Default for LooperLane {
    fn default() -> Self {
        Self::new()
    }
}

impl LooperLane {
    pub fn new() -> Self {
        Self {
            state: LoopState::Empty,
            buffer: None,
            capture_events: Vec::new(),
            pressed: [0; MIDI_OWNERS],
            sustain_down: [false; MIDI_CHANNELS],
            captured_note_on: false,
            playback_cursor_us: None,
            last_transport_beat_us: None,
            accepted_discontinuity_revision: None,
            cleanup_requested: false,
        }
    }

    pub fn state(&self) -> LoopState {
        self.state
    }

    pub fn buffer(&self) -> Option<&LoopBuffer> {
        self.buffer.as_ref()
    }

    pub fn press(
        &mut self,
        now_beats: f64,
        beats_per_bar: u8,
        transport_running: bool,
    ) -> LoopPressOutcome {
        let now = beats_to_us(now_beats);
        let bar = u64::from(beats_per_bar.clamp(1, 32)) * MICROBEATS_PER_BEAT;
        match self.state {
            LoopState::Empty => {
                self.begin_capture();
                let start = if transport_running {
                    next_boundary(now, bar)
                } else {
                    bar
                };
                self.state = LoopState::Armed {
                    start_beat_us: start,
                    beats_per_bar_us: bar,
                };
                if !transport_running {
                    // The adapter performs the requested reset immediately;
                    // do not mistake its beat-zero tick for a backward seek.
                    self.last_transport_beat_us = Some(0);
                }
                LoopPressOutcome {
                    start_transport: !transport_running,
                    reset_transport: !transport_running,
                }
            }
            LoopState::Armed { .. } => {
                self.discard_take();
                LoopPressOutcome::default()
            }
            LoopState::Recording {
                start_beat_us,
                beats_per_bar_us,
                close_beat_us,
            } => {
                if close_beat_us.is_none() {
                    self.state = LoopState::Recording {
                        start_beat_us,
                        beats_per_bar_us,
                        close_beat_us: Some(next_boundary(now, beats_per_bar_us)),
                    };
                }
                LoopPressOutcome::default()
            }
            LoopState::Playing { .. } => {
                self.state = LoopState::Stopped;
                self.playback_cursor_us = None;
                self.cleanup_requested = true;
                LoopPressOutcome::default()
            }
            LoopState::Stopped => {
                if self.buffer.is_none() {
                    self.state = LoopState::Empty;
                    return LoopPressOutcome::default();
                }
                self.start_playback(next_boundary(now, bar));
                LoopPressOutcome {
                    start_transport: !transport_running,
                    reset_transport: false,
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.state = LoopState::Empty;
        self.buffer = None;
        self.begin_capture();
        self.playback_cursor_us = None;
        self.cleanup_requested = true;
    }

    /// Stop safely after reset, seek, meter change, or external transport stop.
    pub fn transport_discontinuity(&mut self) {
        match self.state {
            LoopState::Armed { .. } | LoopState::Recording { .. } => self.discard_take(),
            LoopState::Playing { .. } => {
                self.state = LoopState::Stopped;
                self.playback_cursor_us = None;
            }
            LoopState::Empty | LoopState::Stopped => {}
        }
        self.last_transport_beat_us = None;
        self.cleanup_requested = true;
    }

    pub fn take_cleanup_request(&mut self) -> bool {
        std::mem::take(&mut self.cleanup_requested)
    }

    /// Mark the command-owned reset used for count-in so the router does not
    /// treat that known revision as an external seek.
    pub fn accept_discontinuity_revision(&mut self, revision: u64) {
        self.accepted_discontinuity_revision = Some(revision);
    }

    pub fn take_accepted_discontinuity(&mut self, revision: u64) -> bool {
        if self.accepted_discontinuity_revision == Some(revision) {
            self.accepted_discontinuity_revision = None;
            true
        } else {
            false
        }
    }

    /// Capture only normalized Live input. Loop-origin replay is ignored.
    pub fn capture(&mut self, input: OriginMidiEvent, now_beats: f64) {
        if input.origin != InputOrigin::Live || !input.event.valid() {
            return;
        }
        let now = beats_to_us(now_beats);
        self.advance_capture_state(now);
        let LoopState::Recording {
            start_beat_us,
            beats_per_bar_us,
            close_beat_us,
        } = self.state
        else {
            return;
        };
        if close_beat_us.map(|close| now >= close).unwrap_or(false) {
            self.finish_take(close_beat_us.unwrap());
            return;
        }

        let offset = now.saturating_sub(start_beat_us);
        if offset >= MAX_LOOP_BEATS * MICROBEATS_PER_BEAT {
            let deadline = start_beat_us + MAX_LOOP_BEATS * MICROBEATS_PER_BEAT;
            self.state = LoopState::Recording {
                start_beat_us,
                beats_per_bar_us,
                close_beat_us: close_beat_us
                    .or_else(|| Some(boundary_at_or_after(deadline, beats_per_bar_us))),
            };
            return;
        }
        if self.capture_events.len() >= MAX_LOOP_EVENTS {
            return;
        }

        let event = match input.event {
            LoopMidiEvent::NoteOn {
                note,
                velocity: 0,
                channel,
            } => LoopMidiEvent::NoteOff {
                note,
                velocity: 0,
                channel,
            },
            event => event,
        };
        match event {
            LoopMidiEvent::NoteOn { note, channel, .. } => {
                let owner = owner(note, channel);
                self.pressed[owner] = self.pressed[owner].saturating_add(1);
                self.captured_note_on = true;
            }
            LoopMidiEvent::NoteOff { note, channel, .. } => {
                let owner = owner(note, channel);
                if self.pressed[owner] == 0 {
                    return;
                }
                self.pressed[owner] -= 1;
            }
            LoopMidiEvent::Cc64 { value, channel } => {
                self.sustain_down[channel as usize] = value >= 64;
            }
        }
        self.capture_events.push(LoopEvent {
            beat_offset_us: offset,
            event,
        });
        if self.capture_events.len() == MAX_LOOP_EVENTS {
            self.state = LoopState::Recording {
                start_beat_us,
                beats_per_bar_us,
                close_beat_us: Some(next_boundary(now, beats_per_bar_us)),
            };
        }
    }

    /// Advance state and emit every replay event in `(last_tick, now]` once.
    pub fn tick(&mut self, now_beats: f64) -> Vec<OriginMidiEvent> {
        let now = beats_to_us(now_beats);
        if self
            .last_transport_beat_us
            .map(|last| now < last)
            .unwrap_or(false)
        {
            self.transport_discontinuity();
            self.last_transport_beat_us = Some(now);
            return Vec::new();
        }
        self.last_transport_beat_us = Some(now);
        self.advance_capture_state(now);
        self.replay_until(now)
    }

    pub fn status(&self) -> LoopStatus {
        let (state, close_pending) = match self.state {
            LoopState::Empty => (LoopStatusState::Empty, false),
            LoopState::Armed { .. } => (LoopStatusState::Armed, false),
            LoopState::Recording {
                close_beat_us: Some(_),
                ..
            } => (LoopStatusState::Closing, true),
            LoopState::Recording { .. } => (LoopStatusState::Recording, false),
            LoopState::Playing { .. } => (LoopStatusState::Playing, false),
            LoopState::Stopped => (LoopStatusState::Stopped, false),
        };
        let recorded = self
            .buffer
            .as_ref()
            .map(|buffer| buffer.length_beats_us)
            .unwrap_or(0);
        let recorded_bars = self
            .buffer
            .as_ref()
            .map(|buffer| {
                buffer.length_beats_us as f64 / buffer.recorded_beats_per_bar_us.max(1) as f64
            })
            .unwrap_or(0.0);
        let current = match self.state {
            LoopState::Recording { start_beat_us, .. } => self
                .last_transport_beat_us
                .unwrap_or(start_beat_us)
                .saturating_sub(start_beat_us),
            LoopState::Playing { origin_beat_us } if recorded > 0 => {
                self.last_transport_beat_us
                    .unwrap_or(origin_beat_us)
                    .saturating_sub(origin_beat_us)
                    % recorded
            }
            _ => 0,
        };
        LoopStatus {
            state,
            close_pending,
            has_loop: self.buffer.is_some(),
            current_beats: us_to_beats(current),
            recorded_beats: us_to_beats(recorded),
            recorded_bars,
        }
    }

    fn advance_capture_state(&mut self, now: u64) {
        match self.state {
            LoopState::Armed {
                start_beat_us,
                beats_per_bar_us,
            } if now >= start_beat_us => {
                self.begin_capture();
                self.state = LoopState::Recording {
                    start_beat_us,
                    beats_per_bar_us,
                    close_beat_us: None,
                };
            }
            LoopState::Recording {
                start_beat_us,
                beats_per_bar_us,
                close_beat_us,
            } => {
                let deadline = start_beat_us + MAX_LOOP_BEATS * MICROBEATS_PER_BEAT;
                let close = close_beat_us.or_else(|| {
                    (now >= deadline).then(|| boundary_at_or_after(deadline, beats_per_bar_us))
                });
                if let Some(close) = close {
                    if now >= close {
                        self.finish_take(close);
                    } else if close_beat_us.is_none() {
                        self.state = LoopState::Recording {
                            start_beat_us,
                            beats_per_bar_us,
                            close_beat_us: Some(close),
                        };
                    }
                }
            }
            _ => {}
        }
    }

    fn finish_take(&mut self, close_beat_us: u64) {
        let LoopState::Recording {
            start_beat_us,
            beats_per_bar_us,
            ..
        } = self.state
        else {
            return;
        };
        if !self.captured_note_on || close_beat_us <= start_beat_us {
            self.discard_take();
            return;
        }

        let mut boundary_events = Vec::new();
        for channel in 0..MIDI_CHANNELS {
            for note in 0..MIDI_NOTES {
                let count = self.pressed[channel * MIDI_NOTES + note];
                for _ in 0..count {
                    boundary_events.push(LoopMidiEvent::NoteOff {
                        note: note as u8,
                        velocity: 0,
                        channel: channel as u8,
                    });
                }
            }
        }
        for (channel, down) in self.sustain_down.iter().copied().enumerate() {
            if down {
                boundary_events.push(LoopMidiEvent::Cc64 {
                    value: 0,
                    channel: channel as u8,
                });
            }
        }

        self.buffer = Some(LoopBuffer {
            length_beats_us: close_beat_us - start_beat_us,
            recorded_beats_per_bar_us: beats_per_bar_us,
            events: std::mem::take(&mut self.capture_events),
            boundary_events,
        });
        self.clear_capture_ownership();
        self.start_playback(close_beat_us);
    }

    fn replay_until(&mut self, now: u64) -> Vec<OriginMidiEvent> {
        let LoopState::Playing { origin_beat_us } = self.state else {
            return Vec::new();
        };
        let Some(buffer) = self.buffer.as_ref() else {
            self.state = LoopState::Empty;
            return Vec::new();
        };
        let length = buffer.length_beats_us;
        if length == 0 {
            return Vec::new();
        }
        let cursor = self
            .playback_cursor_us
            .unwrap_or_else(|| origin_beat_us.saturating_sub(1));
        if now <= cursor || now < origin_beat_us {
            return Vec::new();
        }

        let mut due: Vec<(u64, u8, usize, LoopMidiEvent)> = Vec::new();
        for (sequence, event) in buffer.events.iter().enumerate() {
            collect_occurrences(
                &mut due,
                cursor,
                now,
                origin_beat_us + event.beat_offset_us,
                length,
                1,
                sequence,
                event.event,
            );
        }
        for (sequence, event) in buffer.boundary_events.iter().copied().enumerate() {
            collect_occurrences(
                &mut due,
                cursor,
                now,
                origin_beat_us + length,
                length,
                0,
                sequence,
                event,
            );
        }
        due.sort_by_key(|(at, priority, sequence, _)| (*at, *priority, *sequence));
        self.playback_cursor_us = Some(now);
        due.into_iter()
            .map(|(at, _, _, event)| OriginMidiEvent {
                origin: InputOrigin::Loop,
                event,
                scheduled_beat_us: Some(at),
            })
            .collect()
    }

    fn start_playback(&mut self, origin_beat_us: u64) {
        self.state = LoopState::Playing { origin_beat_us };
        self.playback_cursor_us = Some(origin_beat_us.saturating_sub(1));
    }

    fn begin_capture(&mut self) {
        self.capture_events.clear();
        self.clear_capture_ownership();
    }

    fn clear_capture_ownership(&mut self) {
        self.pressed.fill(0);
        self.sustain_down.fill(false);
        self.captured_note_on = false;
    }

    fn discard_take(&mut self) {
        self.state = LoopState::Empty;
        self.buffer = None;
        self.begin_capture();
        self.playback_cursor_us = None;
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_occurrences(
    due: &mut Vec<(u64, u8, usize, LoopMidiEvent)>,
    cursor: u64,
    now: u64,
    first: u64,
    period: u64,
    priority: u8,
    sequence: usize,
    event: LoopMidiEvent,
) {
    let cycle = if cursor < first {
        0
    } else {
        (cursor - first) / period + 1
    };
    let mut at = first.saturating_add(cycle.saturating_mul(period));
    while at <= now {
        due.push((at, priority, sequence, event));
        let next = at.saturating_add(period);
        if next == at {
            break;
        }
        at = next;
    }
}

fn owner(note: u8, channel: u8) -> usize {
    debug_assert!((note as usize) < MIDI_NOTES && (channel as usize) < MIDI_CHANNELS);
    channel as usize * MIDI_NOTES + note as usize
}

fn beats_to_us(beats: f64) -> u64 {
    if !beats.is_finite() || beats <= 0.0 {
        0
    } else {
        (beats * MICROBEATS_PER_BEAT as f64).round() as u64
    }
}

fn us_to_beats(microbeats: u64) -> f64 {
    microbeats as f64 / MICROBEATS_PER_BEAT as f64
}

fn next_boundary(now: u64, beats_per_bar: u64) -> u64 {
    (now / beats_per_bar + 1).saturating_mul(beats_per_bar)
}

fn boundary_at_or_after(beat: u64, beats_per_bar: u64) -> u64 {
    beat.div_ceil(beats_per_bar).saturating_mul(beats_per_bar)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live(event: LoopMidiEvent) -> OriginMidiEvent {
        OriginMidiEvent {
            origin: InputOrigin::Live,
            event,
            scheduled_beat_us: None,
        }
    }

    fn note_on(note: u8, channel: u8) -> OriginMidiEvent {
        live(LoopMidiEvent::NoteOn {
            note,
            velocity: 100,
            channel,
        })
    }

    fn note_off(note: u8, velocity: u8, channel: u8) -> OriginMidiEvent {
        live(LoopMidiEvent::NoteOff {
            note,
            velocity,
            channel,
        })
    }

    fn record_one_bar() -> LooperLane {
        let mut looper = LooperLane::new();
        looper.press(0.1, 4, true);
        looper.tick(4.0);
        looper.capture(note_on(60, 2), 4.25);
        looper.capture(note_off(60, 45, 2), 4.75);
        looper.press(5.0, 4, true);
        looper.tick(8.0);
        looper
    }

    #[test]
    fn default_slot_starts_empty() {
        let looper = LooperLane::new();
        assert_eq!(looper.state(), LoopState::Empty);
        assert_eq!(looper.status().state, LoopStatusState::Empty);
        assert!(looper.buffer().is_none());
    }

    #[test]
    fn running_arm_starts_next_downbeat_and_second_press_closes_next_downbeat() {
        let mut looper = LooperLane::new();
        assert!(!looper.press(1.25, 4, true).start_transport);
        assert_eq!(
            looper.state(),
            LoopState::Armed {
                start_beat_us: 4_000_000,
                beats_per_bar_us: 4_000_000,
            }
        );
        looper.tick(4.0);
        looper.capture(note_on(60, 0), 4.1);
        looper.press(5.2, 4, true);
        assert_eq!(looper.status().state, LoopStatusState::Closing);
        looper.tick(8.0);
        assert!(matches!(looper.state(), LoopState::Playing { .. }));
        assert_eq!(looper.buffer().unwrap().length_beats_us, 4_000_000);
    }

    #[test]
    fn stopped_arm_requests_reset_play_and_one_full_count_in() {
        let mut looper = LooperLane::new();
        let action = looper.press(9.0, 3, false);
        assert!(action.start_transport);
        assert!(action.reset_transport);
        assert_eq!(
            looper.state(),
            LoopState::Armed {
                start_beat_us: 3_000_000,
                beats_per_bar_us: 3_000_000,
            }
        );
        assert!(looper.tick(0.0).is_empty());
        looper.tick(3.0);
        assert!(matches!(looper.state(), LoopState::Recording { .. }));
    }

    #[test]
    fn second_press_while_armed_cancels() {
        let mut looper = LooperLane::new();
        looper.press(0.0, 3, true);
        looper.press(0.1, 3, true);
        assert_eq!(looper.state(), LoopState::Empty);
    }

    #[test]
    fn empty_unmatched_and_malformed_takes_are_discarded() {
        let mut looper = LooperLane::new();
        looper.press(0.0, 4, true);
        looper.tick(4.0);
        looper.capture(note_off(60, 17, 0), 4.5);
        looper.capture(note_off(60, 200, 0), 4.6);
        looper.capture(
            live(LoopMidiEvent::NoteOn {
                note: 61,
                velocity: 200,
                channel: 0,
            }),
            4.7,
        );
        looper.capture(
            live(LoopMidiEvent::Cc64 {
                value: 200,
                channel: 0,
            }),
            4.8,
        );
        looper.press(5.0, 4, true);
        looper.tick(8.0);
        assert_eq!(looper.state(), LoopState::Empty);
        assert!(looper.buffer().is_none());
    }

    #[test]
    fn capture_preserves_velocity_channel_and_cc64_but_rejects_loop_origin() {
        let mut looper = LooperLane::new();
        looper.press(0.0, 4, true);
        looper.tick(4.0);
        looper.capture(
            live(LoopMidiEvent::Cc64 {
                value: 127,
                channel: 3,
            }),
            4.1,
        );
        looper.capture(
            OriginMidiEvent {
                origin: InputOrigin::Loop,
                event: LoopMidiEvent::NoteOn {
                    note: 72,
                    velocity: 1,
                    channel: 9,
                },
                scheduled_beat_us: None,
            },
            4.15,
        );
        looper.capture(
            live(LoopMidiEvent::NoteOn {
                note: 64,
                velocity: 87,
                channel: 3,
            }),
            4.2,
        );
        looper.capture(note_off(64, 23, 3), 4.8);
        looper.capture(
            live(LoopMidiEvent::Cc64 {
                value: 0,
                channel: 3,
            }),
            4.9,
        );
        looper.press(5.0, 4, true);
        looper.tick(8.0);
        let events: Vec<_> = looper
            .buffer()
            .unwrap()
            .events
            .iter()
            .map(|event| event.event)
            .collect();
        assert_eq!(
            events,
            vec![
                LoopMidiEvent::Cc64 {
                    value: 127,
                    channel: 3,
                },
                LoopMidiEvent::NoteOn {
                    note: 64,
                    velocity: 87,
                    channel: 3,
                },
                LoopMidiEvent::NoteOff {
                    note: 64,
                    velocity: 23,
                    channel: 3,
                },
                LoopMidiEvent::Cc64 {
                    value: 0,
                    channel: 3,
                },
            ]
        );
    }

    #[test]
    fn held_repeated_notes_and_sustain_are_closed_at_boundary() {
        let mut looper = LooperLane::new();
        looper.press(0.0, 4, true);
        looper.tick(4.0);
        looper.capture(note_on(60, 2), 4.1);
        looper.capture(note_on(60, 2), 4.2);
        looper.capture(
            live(LoopMidiEvent::Cc64 {
                value: 127,
                channel: 2,
            }),
            4.3,
        );
        looper.press(5.0, 4, true);
        looper.tick(8.0);
        assert_eq!(
            looper.buffer().unwrap().boundary_events,
            vec![
                LoopMidiEvent::NoteOff {
                    note: 60,
                    velocity: 0,
                    channel: 2,
                },
                LoopMidiEvent::NoteOff {
                    note: 60,
                    velocity: 0,
                    channel: 2,
                },
                LoopMidiEvent::Cc64 {
                    value: 0,
                    channel: 2,
                },
            ]
        );
    }

    #[test]
    fn coarse_and_duplicate_ticks_emit_each_due_event_once() {
        let mut looper = record_one_bar();
        let first = looper.tick(9.0);
        assert_eq!(
            first.iter().map(|event| event.event).collect::<Vec<_>>(),
            vec![
                LoopMidiEvent::NoteOn {
                    note: 60,
                    velocity: 100,
                    channel: 2,
                },
                LoopMidiEvent::NoteOff {
                    note: 60,
                    velocity: 45,
                    channel: 2,
                },
            ]
        );
        assert_eq!(
            first
                .iter()
                .map(|event| event.scheduled_beat_us)
                .collect::<Vec<_>>(),
            vec![Some(8_250_000), Some(8_750_000)]
        );
        assert!(looper.tick(9.0).is_empty());
        assert!(looper.tick(11.9).is_empty());
    }

    #[test]
    fn wrap_emits_boundary_cleanup_before_offset_zero_attack() {
        let mut looper = LooperLane::new();
        looper.press(0.0, 4, true);
        looper.tick(4.0);
        looper.capture(note_on(60, 0), 4.0);
        looper.press(5.0, 4, true);
        let start = looper.tick(8.0);
        assert_eq!(start[0].event, note_on(60, 0).event);
        let wrap = looper.tick(12.0);
        assert_eq!(wrap.len(), 2);
        assert!(matches!(wrap[0].event, LoopMidiEvent::NoteOff { .. }));
        assert!(matches!(wrap[1].event, LoopMidiEvent::NoteOn { .. }));
    }

    #[test]
    fn delayed_tick_crosses_multiple_cycles_without_duplicates() {
        let mut looper = record_one_bar();
        let events = looper.tick(20.0);
        assert_eq!(events.len(), 6); // Two captured events across three due cycles.
        assert!(looper.tick(20.0).is_empty());
    }

    #[test]
    fn stop_resume_clear_and_discontinuity_request_cleanup() {
        let mut looper = record_one_bar();
        looper.press(8.1, 4, true);
        assert_eq!(looper.state(), LoopState::Stopped);
        assert!(looper.take_cleanup_request());
        let resume = looper.press(8.1, 4, false);
        assert!(resume.start_transport);
        assert!(!resume.reset_transport);
        assert_eq!(
            looper.state(),
            LoopState::Playing {
                origin_beat_us: 12_000_000,
            }
        );
        looper.transport_discontinuity();
        assert_eq!(looper.state(), LoopState::Stopped);
        assert!(looper.buffer().is_some());
        assert!(looper.take_cleanup_request());
        looper.clear();
        assert_eq!(looper.state(), LoopState::Empty);
        assert!(looper.buffer().is_none());
    }

    #[test]
    fn backward_tick_stops_playback_and_retains_buffer() {
        let mut looper = record_one_bar();
        looper.tick(9.0);
        assert!(looper.tick(2.0).is_empty());
        assert_eq!(looper.state(), LoopState::Stopped);
        assert!(looper.buffer().is_some());
        assert!(looper.take_cleanup_request());
    }

    #[test]
    fn maximum_duration_is_derived_from_start_not_late_poll_time() {
        let mut looper = LooperLane::new();
        looper.press(0.0, 4, true);
        looper.tick(4.0);
        looper.capture(note_on(60, 0), 4.1);
        looper.tick(1_000.0);
        assert_eq!(looper.buffer().unwrap().length_beats_us, 256_000_000);
    }

    #[test]
    fn event_ceiling_requests_close_on_the_16384th_event() {
        let mut looper = LooperLane::new();
        looper.press(0.0, 4, true);
        looper.tick(4.0);
        for _ in 0..(MAX_LOOP_EVENTS / 2) {
            looper.capture(note_on(60, 0), 4.1);
            looper.capture(note_off(60, 0, 0), 4.1);
        }
        assert_eq!(looper.status().state, LoopStatusState::Closing);
        looper.capture(note_on(62, 0), 4.2); // Must not move the close boundary.
        looper.tick(8.0);
        assert_eq!(looper.buffer().unwrap().length_beats_us, 4_000_000);
    }
}
