//! Counterpoint Lane — a dedicated Fux-style species-counterpoint voice
//! that sits alongside the CanonLane under the same Companion master.
//!
//! Unlike the canon's per-voice mini-engine which is 1-input → 1-output
//! and only really delivers Species 1, this Lane subdivides time per
//! species:
//!
//! * Species 1 (note-against-note): emits one harmony note per cantus note.
//! * Species 2 (2:1): two emissions — one at the cantus note's onset
//!   (strong-beat consonance) and one at the half-beat (weak-beat
//!   passing or auxiliary).
//! * Species 3 (4:1): four emissions per cantus note at 0 / 0.25 / 0.5 /
//!   0.75 of the beat (strong + three passing).
//! * Species 4 (syncopated suspensions): the counterpoint enters half a
//!   beat after each cantus note, creating ties across barlines and
//!   suspension/resolution on strong beats.
//!
//! Each species reuses the existing `CounterpointState` for choosing
//! the strong-beat pitch (interval rules, parallel-5th avoidance, etc.)
//! and adds its own scheduling logic on top.
//!
//! v1 (this file): Species 1 + 2 fully wired. Species 3/4 land in
//! follow-up commits.

#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

use wmidi::Note;

use contrapunk::harmony::{CounterpointSpecies, CounterpointState, HarmonyMode, Key, ScaleMode};

use super::lane::{InputEvent, InputFilter, Lane, LaneOutput, LanePhase};
use super::world::WorldState;
use super::DispatchOp;
use crate::state::VoiceOutputTarget;

/// One pending counterpoint NoteOn scheduled for a future beat.
#[derive(Clone, Copy, Debug)]
struct PendingCpOn {
    fire_at: f64,
    note: u8,
    velocity: u8,
    channel: u8,
    /// Tag identifying which player note this emission belongs to —
    /// matched on NoteOff so the right pitches release together.
    player_note: u8,
}

/// One pending NoteOff.
#[derive(Clone, Copy, Debug)]
struct PendingCpOff {
    fire_at: f64,
    note: u8,
    channel: u8,
}

/// Held entry tracking which emitted pitches a given player NoteOn
/// is responsible for. Required because Species 2/3 emit MORE THAN ONE
/// note per cantus note, so a single NoteOff must release all of them.
#[derive(Clone, Debug)]
struct HeldCpEntry {
    on_beat: f64,
    emitted_notes: Vec<u8>,
    channel: u8,
}

pub struct CounterpointLane {
    pub enabled: bool,
    pub species: CounterpointSpecies,
    /// Diatonic transpose for the counterpoint line's first emission —
    /// e.g. +2 puts the line a third above the cantus. Clamped [-7, 7].
    pub transpose_degrees: i8,
    /// Direction preference for chained pitches: true = harmony above
    /// the cantus, false = below. Reflected to the underlying engine
    /// rules via `process_directed`.
    pub prefer_above: bool,
    /// Where this voice routes (synth / midi port / off).
    pub voice_output: VoiceOutputTarget,

    state: CounterpointState,
    /// Cantus firmus history — last N input notes with their beat
    /// positions. Used by Species 4 suspensions which need to know
    /// what was on the previous strong beat.
    cantus_history: VecDeque<(Note, f64)>,
    /// Pending emissions, drained by `tick` when the transport's beat
    /// catches up. Sorted by fire_at ascending.
    pending_on: VecDeque<PendingCpOn>,
    pending_off: VecDeque<PendingCpOff>,
    /// Player-note → emitted pitches map, so NoteOff releases the right
    /// set when Species 2/3 has emitted multiple pitches per input.
    held: HashMap<u8, HeldCpEntry>,
}

impl CounterpointLane {
    pub fn new() -> Self {
        Self {
            enabled: false,
            species: CounterpointSpecies::Species1,
            transpose_degrees: 2, // default: a third above
            prefer_above: true,
            voice_output: VoiceOutputTarget::Synth,
            state: CounterpointState::new(),
            cantus_history: VecDeque::with_capacity(8),
            pending_on: VecDeque::new(),
            pending_off: VecDeque::new(),
            held: HashMap::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.pending_on.clear();
            self.pending_off.clear();
            self.held.clear();
            self.state.reset();
            self.cantus_history.clear();
        }
    }

    pub fn set_species(&mut self, species: CounterpointSpecies) {
        if self.species != species {
            self.species = species;
            // Resetting the state on species change so the new species
            // doesn't inherit history that was scored under the old
            // species' rules.
            self.state.reset();
            self.state.set_species(species);
        }
    }

    /// Compute the harmony pitch for the player's input using the
    /// scoring rules baked into CounterpointState.
    fn pick_pitch(&mut self, scale: &mut contrapunk::harmony::Scale, melody: Note) -> Option<Note> {
        // process_directed updates state and returns [melody, harmony].
        let result = self
            .state
            .process_directed(scale, melody, self.prefer_above);
        if result.len() > 1 {
            Some(result[1])
        } else {
            None
        }
    }

    /// Pick a stepwise neighbour pitch (one diatonic step away from
    /// `from`) for use as a weak-beat passing/auxiliary tone in
    /// Species 2/3. Falls back to `from` if the scale's bounds reject
    /// the step.
    fn pick_passing(
        &self,
        scale: &mut contrapunk::harmony::Scale,
        from: Note,
        direction: i8,
    ) -> Note {
        scale.transpose_diatonic(from, direction).unwrap_or(from)
    }

    fn insert_pending(&mut self, p: PendingCpOn) {
        let pos = self
            .pending_on
            .iter()
            .rposition(|existing| existing.fire_at <= p.fire_at);
        match pos {
            Some(idx) => self.pending_on.insert(idx + 1, p),
            None => self.pending_on.push_front(p),
        }
    }
}

impl Default for CounterpointLane {
    fn default() -> Self {
        Self::new()
    }
}

impl Lane for CounterpointLane {
    fn name(&self) -> &str {
        "Counterpoint"
    }
    fn type_id(&self) -> &str {
        "counterpoint"
    }
    fn phase(&self) -> LanePhase {
        LanePhase::Decide
    }
    fn input_filter(&self) -> InputFilter {
        InputFilter::All
    }

    fn on_input(&mut self, ev: InputEvent, world: &WorldState) -> LaneOutput {
        if !self.enabled {
            return LaneOutput::default();
        }
        let now = world.transport.total_beats();

        match ev {
            InputEvent::NoteOn {
                note,
                velocity,
                channel,
            } => {
                // Snapshot key/scale_mode under the lock to feed the
                // scoring scale. State stays per-lane.
                let (key, scale_mode) = if let Ok(g) = world.engine_snapshot.lock() {
                    (g.key(), g.scale_mode())
                } else {
                    (Key::C, ScaleMode::Ionian)
                };
                let mut scale = contrapunk::harmony::Scale::new(key.semitones_from_c(), scale_mode);
                let Ok(melody_note) = Note::try_from(note) else {
                    return LaneOutput::default();
                };

                // Apply diatonic transpose to the starting reference so
                // the counterpoint sits in a configured interval band.
                let transposed = if self.transpose_degrees != 0 {
                    scale
                        .harmonize_smart(melody_note, self.transpose_degrees, self.prefer_above)
                        .unwrap_or(melody_note)
                } else {
                    melody_note
                };

                let primary = self
                    .pick_pitch(&mut scale, transposed)
                    .unwrap_or(transposed);
                let primary_midi = u8::from(primary);

                // Track cantus history for Species 4 future use.
                if self.cantus_history.len() >= 8 {
                    self.cantus_history.pop_front();
                }
                self.cantus_history.push_back((melody_note, now));

                // Schedule emissions per species.
                let mut emitted_notes: Vec<u8> = Vec::new();
                match self.species {
                    CounterpointSpecies::Species1 => {
                        // 1 emission, on the cantus onset.
                        self.insert_pending(PendingCpOn {
                            fire_at: now,
                            note: primary_midi,
                            velocity,
                            channel,
                            player_note: note,
                        });
                        emitted_notes.push(primary_midi);
                    }
                    CounterpointSpecies::Species2 => {
                        // Strong beat: primary consonance at onset.
                        self.insert_pending(PendingCpOn {
                            fire_at: now,
                            note: primary_midi,
                            velocity,
                            channel,
                            player_note: note,
                        });
                        emitted_notes.push(primary_midi);
                        // Weak beat: passing tone half a beat later,
                        // stepping toward the next likely consonance.
                        let direction: i8 = if self.prefer_above { -1 } else { 1 };
                        let passing = self.pick_passing(&mut scale, primary, direction);
                        let passing_midi = u8::from(passing);
                        self.insert_pending(PendingCpOn {
                            fire_at: now + 0.5,
                            note: passing_midi,
                            velocity: velocity.saturating_sub(20).max(60),
                            channel,
                            player_note: note,
                        });
                        emitted_notes.push(passing_midi);
                    }
                    CounterpointSpecies::Species3 => {
                        // 4:1 — strong beat consonance + 3 passing
                        // tones at quarter-beat subdivisions.
                        self.insert_pending(PendingCpOn {
                            fire_at: now,
                            note: primary_midi,
                            velocity,
                            channel,
                            player_note: note,
                        });
                        emitted_notes.push(primary_midi);
                        let direction: i8 = if self.prefer_above { -1 } else { 1 };
                        let mut cursor = primary;
                        for (i, offset) in [0.25, 0.5, 0.75].iter().enumerate() {
                            let next = self.pick_passing(&mut scale, cursor, direction);
                            cursor = next;
                            let m = u8::from(next);
                            self.insert_pending(PendingCpOn {
                                fire_at: now + offset,
                                note: m,
                                velocity: velocity.saturating_sub(15 + i as u8 * 5).max(50),
                                channel,
                                player_note: note,
                            });
                            emitted_notes.push(m);
                        }
                    }
                    CounterpointSpecies::Species4 => {
                        // Syncopated: emission lands half a beat later
                        // than the cantus onset. When the next cantus
                        // note arrives, the held pitch is the
                        // "suspension". v1: just the half-beat delay;
                        // proper resolution machinery follows in the
                        // next slice.
                        self.insert_pending(PendingCpOn {
                            fire_at: now + 0.5,
                            note: primary_midi,
                            velocity,
                            channel,
                            player_note: note,
                        });
                        emitted_notes.push(primary_midi);
                    }
                }

                self.held.insert(
                    note,
                    HeldCpEntry {
                        on_beat: now,
                        emitted_notes,
                        channel,
                    },
                );
            }
            InputEvent::NoteOff { note, channel: _ } => {
                if let Some(held) = self.held.remove(&note) {
                    // Off-time per emitted note: hold each emission for
                    // the same duration the cantus note was held,
                    // shifted forward from its scheduled on-time. We
                    // don't know the cantus duration until NoteOff so
                    // approximate by firing all offs at `now` (the
                    // cantus release moment).
                    for n in held.emitted_notes {
                        self.pending_off.push_back(PendingCpOff {
                            fire_at: now,
                            note: n,
                            channel: held.channel,
                        });
                    }
                }
            }
            InputEvent::Cc { .. } => {}
        }

        LaneOutput::default()
    }

    fn tick(&mut self, world: &WorldState) -> LaneOutput {
        if !self.enabled {
            return LaneOutput::default();
        }
        let now = world.transport.total_beats();
        let mut ops: Vec<DispatchOp> = Vec::new();

        while let Some(front) = self.pending_on.front() {
            if front.fire_at > now {
                break;
            }
            let p = self.pending_on.pop_front().unwrap();
            ops.push(DispatchOp::NoteOn {
                target: self.voice_output,
                note: p.note,
                velocity: p.velocity,
                channel: p.channel,
            });
        }

        // Off queue isn't kept sorted (small N, infrequent), so scan
        // linearly. Retain anything still in the future.
        let mut idx = 0;
        while idx < self.pending_off.len() {
            if self.pending_off[idx].fire_at <= now {
                let p = self.pending_off.remove(idx).unwrap();
                ops.push(DispatchOp::NoteOff {
                    target: self.voice_output,
                    note: p.note,
                    channel: p.channel,
                });
            } else {
                idx += 1;
            }
        }

        LaneOutput {
            ops,
            ..Default::default()
        }
    }

    fn serialize_state(&self) -> serde_json::Value {
        let species_str = match self.species {
            CounterpointSpecies::Species1 => "Species1",
            CounterpointSpecies::Species2 => "Species2",
            CounterpointSpecies::Species3 => "Species3",
            CounterpointSpecies::Species4 => "Species4",
        };
        serde_json::json!({
            "enabled": self.enabled,
            "species": species_str,
            "transpose_degrees": self.transpose_degrees,
            "prefer_above": self.prefer_above,
        })
    }

    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
        if let Some(b) = state.get("enabled").and_then(|v| v.as_bool()) {
            self.set_enabled(b);
        }
        if let Some(s) = state.get("species").and_then(|v| v.as_str()) {
            let species = match s {
                "Species1" => CounterpointSpecies::Species1,
                "Species2" => CounterpointSpecies::Species2,
                "Species3" => CounterpointSpecies::Species3,
                "Species4" => CounterpointSpecies::Species4,
                _ => return Err(format!("unknown counterpoint species '{}'", s)),
            };
            self.set_species(species);
        }
        if let Some(t) = state.get("transpose_degrees").and_then(|v| v.as_i64()) {
            self.transpose_degrees = (t as i8).clamp(-7, 7);
        }
        if let Some(p) = state.get("prefer_above").and_then(|v| v.as_bool()) {
            self.prefer_above = p;
        }
        Ok(())
    }
}

// Suppress unused-field warnings on HarmonyMode (referenced via the
// scale module but not directly used by this file's symbols today).
const _: Option<HarmonyMode> = None;
