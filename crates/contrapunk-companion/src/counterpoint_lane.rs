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
//! * Species 4 (syncopated suspensions): prepares a consonance on the
//!   half-beat, retains that exact note over the next strong beat, and
//!   resolves a valid live suspension down one diatonic step on the
//!   following half-beat.
//!
//! Species 1-3 reuse `CounterpointState` for strong-beat pitch choice.
//! Species 4 owns one explicit gesture because a held MIDI note cannot
//! be represented safely as unrelated pending attacks. Its opt-in phrase
//! policy adds one coordinated bass beneath that tied voice.

#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

use wmidi::Note;

use contrapunk_harmony::{CounterpointSpecies, CounterpointState, HarmonyMode, Key, ScaleMode};

use super::lane::{
    hold_mode_from_json, hold_mode_to_json, HoldMode, InputEvent, InputFilter, Lane, LaneOutput,
    LanePhase,
};
use super::phrase::PhrasePhase;
use super::world::WorldState;
use super::DispatchOp;
use crate::voice_output::VoiceOutputTarget;

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

const SPECIES4_STRONG_WINDOW: f64 = 0.2;
const SPECIES4_LATE_PREPARATION: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Species4Stage {
    Armed,
    Prepared,
    Suspended {
        resolution: u8,
        resolve_at: f64,
        bass_target: u8,
    },
    Resolved {
        release_at: f64,
    },
}

/// One fourth-species gesture. `pitch` is the tied suspension voice;
/// phrase-shaped gestures also own one coordinated `bass_pitch`.
/// `Prepared` and `Suspended` mean both have sounded; `Armed` is silent.
#[derive(Clone, Copy, Debug)]
struct Species4Gesture {
    owner_note: u8,
    owner_channel: u8,
    pitch: u8,
    bass_pitch: Option<u8>,
    velocity: u8,
    prepare_at: f64,
    strong_at: f64,
    release_by: Option<f64>,
    stage: Species4Stage,
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
    /// Shape Species 4 as a bounded phrase: expose the opening attack,
    /// coordinate a bass beneath the tied voice, breathe after resolutions,
    /// and stop after two successful gestures. Off preserves legacy behavior.
    pub phrase_aware: bool,
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
    /// One logical gesture owns the tied voice and optional phrase bass.
    species4: Option<Species4Gesture>,
    /// Bass cleanup deferred from input handling to the slotted tick path.
    species4_bass_cleanup: Option<(u8, u8)>,
    species4_phrase_id: Option<u64>,
    species4_completed: u8,
    species4_breathe_until: Option<f64>,
    last_tick_beat: Option<f64>,
    /// Lane-level HoldMode override. `None` = inherit Companion global.
    /// Applies to every pending emission seeded by a player NoteOn when
    /// that player note is released. See `lane::HoldMode` for the four
    /// modes (Cancel / NearFuture / PhraseEnd / Forever).
    pub hold_mode: Option<HoldMode>,
}

impl CounterpointLane {
    pub fn new() -> Self {
        Self {
            // FTUX default: ON, alongside the canon. The Companion master
            // toggle still gates everything, so a user who wants quiet
            // can switch the Companion off; from the lane's perspective
            // the goal is "fresh install plays counterpoint from note one".
            enabled: true,
            species: CounterpointSpecies::Species1,
            transpose_degrees: 2, // default: a third above
            prefer_above: true,
            phrase_aware: false,
            voice_output: VoiceOutputTarget::Synth,
            state: CounterpointState::new(),
            cantus_history: VecDeque::with_capacity(8),
            pending_on: VecDeque::new(),
            pending_off: VecDeque::new(),
            held: HashMap::new(),
            species4: None,
            species4_bass_cleanup: None,
            species4_phrase_id: None,
            species4_completed: 0,
            species4_breathe_until: None,
            last_tick_beat: None,
            hold_mode: None,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled && !enabled {
            self.pending_on.clear();
            self.pending_off.clear();
            self.flush_species4();
            self.held.clear();
            self.state.reset();
            self.cantus_history.clear();
            self.reset_species4_phrase();
        }
        self.enabled = enabled;
    }

    pub fn set_species(&mut self, species: CounterpointSpecies) {
        if self.species != species {
            if self.species == CounterpointSpecies::Species4 {
                self.flush_species4();
                self.cantus_history.clear();
            }
            self.reset_species4_phrase();
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
    fn pick_pitch(&mut self, scale: &mut contrapunk_harmony::Scale, melody: Note) -> Option<Note> {
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
        scale: &mut contrapunk_harmony::Scale,
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

    fn is_consonant(a: u8, b: u8) -> bool {
        matches!(a.abs_diff(b) % 12, 0 | 3 | 4 | 7 | 8 | 9)
    }

    fn is_species4_dissonance(a: u8, b: u8) -> bool {
        matches!(a.abs_diff(b) % 12, 1 | 2 | 5 | 10 | 11)
    }

    fn consonance_quality(a: u8, b: u8) -> u8 {
        match a.abs_diff(b) % 12 {
            3 | 4 | 8 | 9 => 0,
            7 => 1,
            0 => 2,
            _ => u8::MAX,
        }
    }

    fn species4_preparation_bass(
        scale: &mut contrapunk_harmony::Scale,
        melody: Note,
        suspension: Note,
    ) -> Option<Note> {
        let anchor = scale.snap_to_scale(melody);
        let melody_midi = u8::from(melody);
        let suspension_midi = u8::from(suspension);
        let ideal = melody_midi.saturating_sub(12);
        let mut best: Option<(u8, u8, Note)> = None;
        for degrees in -14i8..=-3i8 {
            let Some(candidate) = scale.transpose_diatonic(anchor, degrees) else {
                continue;
            };
            let bass = u8::from(candidate);
            if bass > suspension_midi.saturating_sub(7)
                || !Self::is_consonant(bass, melody_midi)
                || !Self::is_consonant(bass, suspension_midi)
            {
                continue;
            }
            let score = (
                Self::consonance_quality(bass, melody_midi)
                    .saturating_add(Self::consonance_quality(bass, suspension_midi)),
                bass.abs_diff(ideal),
                candidate,
            );
            if best.map(|current| score < current).unwrap_or(true) {
                best = Some(score);
            }
        }
        best.map(|(_, _, note)| note)
    }

    fn species4_suspension_bass(
        scale: &mut contrapunk_harmony::Scale,
        melody: Note,
        suspension: Note,
        resolution: Note,
        previous_bass: u8,
    ) -> Option<Note> {
        let anchor = scale.snap_to_scale(melody);
        let melody_midi = u8::from(melody);
        let suspension_midi = u8::from(suspension);
        let resolution_midi = u8::from(resolution);
        if !Self::is_consonant(melody_midi, resolution_midi) {
            return None;
        }
        let mut best: Option<(u8, u8, Note)> = None;
        for degrees in -14i8..=-3i8 {
            let Some(candidate) = scale.transpose_diatonic(anchor, degrees) else {
                continue;
            };
            let bass = u8::from(candidate);
            if bass == previous_bass
                || bass > suspension_midi.saturating_sub(7)
                || !Self::is_species4_dissonance(suspension_midi, bass)
                || !Self::is_consonant(resolution_midi, bass)
                || !Self::is_consonant(melody_midi, bass)
            {
                continue;
            }
            let score = (
                Self::consonance_quality(resolution_midi, bass)
                    .saturating_add(Self::consonance_quality(melody_midi, bass)),
                bass.abs_diff(previous_bass),
                candidate,
            );
            if best.map(|current| score < current).unwrap_or(true) {
                best = Some(score);
            }
        }
        best.map(|(_, _, note)| note)
    }

    fn next_weak_beat(now: f64) -> f64 {
        let half = now.floor() + 0.5;
        if now <= half {
            half
        } else {
            half + 1.0
        }
    }

    fn species4_preparation(
        &mut self,
        scale: &mut contrapunk_harmony::Scale,
        melody: Note,
    ) -> Option<Note> {
        let configured = if self.transpose_degrees == 0 {
            Some(melody)
        } else {
            scale.harmonize_smart(melody, self.transpose_degrees, self.prefer_above)
        };
        configured
            .filter(|candidate| Self::is_consonant(u8::from(melody), u8::from(*candidate)))
            .or_else(|| self.pick_pitch(scale, melody))
            .filter(|candidate| Self::is_consonant(u8::from(melody), u8::from(*candidate)))
    }

    fn arm_species4(
        &mut self,
        scale: &mut contrapunk_harmony::Scale,
        melody: Note,
        velocity: u8,
        channel: u8,
        now: f64,
    ) {
        let Some(preparation) = self.species4_preparation(scale, melody) else {
            self.species4 = None;
            return;
        };
        let bass_pitch = if self.phrase_aware {
            let Some(bass) = Self::species4_preparation_bass(scale, melody, preparation) else {
                self.species4 = None;
                return;
            };
            Some(u8::from(bass))
        } else {
            None
        };
        let prepare_at = Self::next_weak_beat(now);
        self.species4 = Some(Species4Gesture {
            owner_note: u8::from(melody),
            owner_channel: channel,
            pitch: u8::from(preparation),
            bass_pitch,
            velocity,
            prepare_at,
            strong_at: prepare_at + 0.5,
            release_by: None,
            stage: Species4Stage::Armed,
        });
    }

    fn on_species4_note_on(
        &mut self,
        scale: &mut contrapunk_harmony::Scale,
        melody: Note,
        velocity: u8,
        channel: u8,
        now: f64,
    ) -> LaneOutput {
        let Some(mut gesture) = self.species4.take() else {
            self.arm_species4(scale, melody, velocity, channel, now);
            return LaneOutput::default();
        };

        let mut ops = Vec::new();
        match gesture.stage {
            Species4Stage::Armed => {
                // The preparation has not sounded, so the newest live
                // note can replace it without any cleanup.
                self.arm_species4(scale, melody, velocity, channel, now);
            }
            Species4Stage::Prepared => {
                let melody_midi = u8::from(melody);
                let resolution = Note::try_from(gesture.pitch)
                    .ok()
                    .and_then(|pitch| scale.transpose_diatonic(pitch, -1));

                if self.phrase_aware {
                    let target = resolution.and_then(|resolution| {
                        let previous_bass = gesture.bass_pitch?;
                        Self::species4_suspension_bass(
                            scale,
                            melody,
                            Note::try_from(gesture.pitch).ok()?,
                            resolution,
                            previous_bass,
                        )
                        .map(|bass| (resolution, bass))
                    });
                    if channel == gesture.owner_channel {
                        if let Some((resolution, bass_target)) = target {
                            gesture.owner_note = melody_midi;
                            gesture.velocity = velocity;
                            gesture.release_by = None;
                            gesture.stage = Species4Stage::Suspended {
                                resolution: u8::from(resolution),
                                resolve_at: now + 0.5,
                                bass_target: u8::from(bass_target),
                            };
                            self.species4 = Some(gesture);
                            return LaneOutput::default();
                        }
                    }
                    self.queue_phrase_species4_cleanup(gesture);
                    self.arm_species4(scale, melody, velocity, channel, now);
                } else {
                    let on_expected_strong =
                        (now - gesture.strong_at).abs() <= SPECIES4_STRONG_WINDOW;
                    let consonant = Self::is_consonant(gesture.pitch, melody_midi);
                    let resolution =
                        resolution.filter(|note| Self::is_consonant(u8::from(*note), melody_midi));
                    match (
                        on_expected_strong && channel == gesture.owner_channel,
                        consonant,
                        resolution,
                    ) {
                        (true, false, Some(resolution))
                            if Self::is_species4_dissonance(gesture.pitch, melody_midi) =>
                        {
                            gesture.owner_note = melody_midi;
                            gesture.velocity = velocity;
                            gesture.release_by = None;
                            gesture.stage = Species4Stage::Suspended {
                                resolution: u8::from(resolution),
                                resolve_at: gesture.strong_at + 0.5,
                                bass_target: 0,
                            };
                            self.species4 = Some(gesture);
                        }
                        (true, true, _) => {
                            // A legal consonant syncopation, not a suspension.
                            // Keep it as the preparation for the next strong beat.
                            gesture.owner_note = melody_midi;
                            gesture.velocity = velocity;
                            gesture.release_by = None;
                            gesture.strong_at += 1.0;
                            self.species4 = Some(gesture);
                        }
                        _ => {
                            // Release before the new live NoteOn is dispatched;
                            // an arbitrary unprepared dissonance must never sound.
                            ops.push(DispatchOp::NoteOff {
                                target: self.voice_output,
                                note: gesture.pitch,
                                channel: gesture.owner_channel,
                            });
                            self.arm_species4(scale, melody, velocity, channel, now);
                        }
                    }
                }
            }
            Species4Stage::Suspended { .. } => {
                // Another melody change before the planned resolution
                // invalidates the pre-checked sonority.
                if self.phrase_aware {
                    self.queue_phrase_species4_cleanup(gesture);
                } else {
                    ops.push(DispatchOp::NoteOff {
                        target: self.voice_output,
                        note: gesture.pitch,
                        channel: gesture.owner_channel,
                    });
                }
                self.arm_species4(scale, melody, velocity, channel, now);
            }
            Species4Stage::Resolved { .. } => {
                // Phrase-shaped Species 4 lets the resolution finish before
                // accepting another preparation.
                self.species4 = Some(gesture);
            }
        }

        LaneOutput {
            ops,
            ..Default::default()
        }
    }

    fn queue_phrase_species4_cleanup(&mut self, gesture: Species4Gesture) {
        if gesture.stage == Species4Stage::Armed {
            return;
        }
        self.pending_off.push_back(PendingCpOff {
            fire_at: 0.0,
            note: gesture.pitch,
            channel: gesture.owner_channel,
        });
        if let Some(bass) = gesture.bass_pitch {
            self.species4_bass_cleanup = Some((bass, gesture.owner_channel));
        }
    }

    fn flush_species4(&mut self) {
        let Some(gesture) = self.species4.take() else {
            return;
        };
        if self.phrase_aware {
            self.queue_phrase_species4_cleanup(gesture);
        } else if gesture.stage != Species4Stage::Armed {
            self.pending_off.push_back(PendingCpOff {
                fire_at: 0.0,
                note: gesture.pitch,
                channel: gesture.owner_channel,
            });
        }
    }

    fn reset_species4_phrase(&mut self) {
        self.species4_phrase_id = None;
        self.species4_completed = 0;
        self.species4_breathe_until = None;
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
    fn type_id(&self) -> &'static str {
        "counterpoint"
    }
    fn phase(&self) -> LanePhase {
        LanePhase::Decide
    }
    fn input_filter(&self) -> InputFilter {
        InputFilter::All
    }

    fn reset_runtime(&mut self) {
        self.pending_on.clear();
        self.pending_off.clear();
        self.held.clear();
        self.species4 = None;
        self.species4_bass_cleanup = None;
        self.reset_species4_phrase();
        self.last_tick_beat = None;
        self.state.reset();
        self.cantus_history.clear();
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
                let mut scale = contrapunk_harmony::Scale::new(key.semitones_from_c(), scale_mode);
                let Ok(melody_note) = Note::try_from(note) else {
                    return LaneOutput::default();
                };

                // Species 4 owns a single transport-scheduled gesture;
                // do not mix it with the generic pending/held ledger.
                if self.species == CounterpointSpecies::Species4 {
                    if self.phrase_aware {
                        let phrase = world.phrase_snapshot();
                        if phrase.id != self.species4_phrase_id {
                            if let Some(gesture) = self.species4.take() {
                                self.queue_phrase_species4_cleanup(gesture);
                            }
                            self.species4_phrase_id = phrase.id;
                            self.species4_completed = 0;
                            self.species4_breathe_until = None;
                            self.cantus_history.clear();
                        }
                        let breathing = self
                            .species4_breathe_until
                            .map(|until| now < until)
                            .unwrap_or(false);
                        if phrase.attack_count <= 1 || self.species4_completed >= 2 || breathing {
                            return LaneOutput::default();
                        }
                    }
                    if self.cantus_history.len() >= 8 {
                        self.cantus_history.pop_front();
                    }
                    self.cantus_history.push_back((melody_note, now));
                    return self.on_species4_note_on(
                        &mut scale,
                        melody_note,
                        velocity,
                        channel,
                        now,
                    );
                }

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
                    CounterpointSpecies::Species4 => unreachable!("handled above"),
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
            InputEvent::NoteOff { note, channel } => {
                // Species 4 transfers ownership on a new strong-beat
                // NoteOn before the old legato NoteOff arrives.
                if self.species == CounterpointSpecies::Species4 {
                    let Some(mut gesture) = self.species4.take() else {
                        return LaneOutput::default();
                    };
                    if gesture.owner_note != note || gesture.owner_channel != channel {
                        self.species4 = Some(gesture);
                        return LaneOutput::default();
                    }
                    if self.phrase_aware && !world.phrase_snapshot().input_idle {
                        gesture.release_by = None;
                        self.species4 = Some(gesture);
                        return LaneOutput::default();
                    }

                    let global_hold = world
                        .global_hold_mode
                        .lock()
                        .map(|g| *g)
                        .unwrap_or_default();
                    let effective = self.hold_mode.unwrap_or(global_hold);
                    let beats_per_bar = world.transport.time_signature().0 as f64;
                    let release_by = match effective {
                        HoldMode::Cancel => None,
                        HoldMode::NearFuture { tail_beats } => Some(now + tail_beats),
                        HoldMode::PhraseEnd => {
                            Some((now / beats_per_bar).floor() * beats_per_bar + beats_per_bar)
                        }
                        HoldMode::Forever => Some(match gesture.stage {
                            Species4Stage::Suspended { resolve_at, .. } => resolve_at + 1.0,
                            _ => gesture.strong_at + 0.5,
                        }),
                    };

                    if effective == HoldMode::Cancel {
                        if self.phrase_aware {
                            self.queue_phrase_species4_cleanup(gesture);
                            return LaneOutput::default();
                        }
                        let ops = if gesture.stage == Species4Stage::Armed {
                            Vec::new()
                        } else {
                            vec![DispatchOp::NoteOff {
                                target: self.voice_output,
                                note: gesture.pitch,
                                channel: gesture.owner_channel,
                            }]
                        };
                        return LaneOutput {
                            ops,
                            ..Default::default()
                        };
                    }
                    gesture.release_by = release_by;
                    self.species4 = Some(gesture);
                    return LaneOutput::default();
                }

                // Resolve effective HoldMode (lane override > global).
                // Pre-v1.2 CounterpointLane behavior was effectively
                // HoldMode::Cancel (it dropped all pending NoteOns and
                // released emitted notes at `now`). That stays the
                // default *behavior* if mode resolves to Cancel — the
                // FTUX continues to feel performative. Users who want
                // canon-style sustained passing tones flip the toggle
                // to Forever or NearFuture/PhraseEnd.
                let global_hold = world
                    .global_hold_mode
                    .lock()
                    .map(|g| *g)
                    .unwrap_or_default();
                let effective = self.hold_mode.unwrap_or(global_hold);
                let beats_per_bar = world.transport.time_signature().0 as f64;
                // PhraseEnd horizon: same idea as CanonLane — let
                // pending emissions within the current bar fire.
                let phrase_end = (now / beats_per_bar).floor() * beats_per_bar + beats_per_bar;

                // Decide which pending NoteOns for this player note
                // get cancelled vs let-fire, per the effective mode.
                let mut canceled_pitches: Vec<u8> = Vec::new();
                self.pending_on.retain(|p| {
                    if p.player_note != note {
                        return true;
                    }
                    let keep = match effective {
                        HoldMode::Forever => true,
                        HoldMode::Cancel => false,
                        HoldMode::NearFuture { tail_beats } => (p.fire_at - now) <= tail_beats,
                        HoldMode::PhraseEnd => p.fire_at <= phrase_end,
                    };
                    if !keep {
                        canceled_pitches.push(p.note);
                    }
                    keep
                });
                if let Some(mut held) = self.held.remove(&note) {
                    // Cancelled NoteOns get no matching NoteOff — they
                    // never produced sound. Already-emitted notes get
                    // released at `now`, OR at `now + tail_beats` so
                    // the lookahead / sustain semantics feel coherent.
                    held.emitted_notes.retain(|n| !canceled_pitches.contains(n));
                    let release_at = match effective {
                        HoldMode::NearFuture { tail_beats } => now + tail_beats,
                        _ => now,
                    };
                    for n in held.emitted_notes {
                        self.pending_off.push_back(PendingCpOff {
                            fire_at: release_at,
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
        let now = world.transport.total_beats();
        let mut ops: Vec<DispatchOp> = Vec::new();

        // Cleanup queued by disable/species changes must drain even
        // after the lane gate closes.
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
        if let Some((note, channel)) = self.species4_bass_cleanup.take() {
            ops.push(DispatchOp::NoteOff {
                target: self.voice_output,
                note,
                channel,
            });
        }

        // A host seek/loop can move total_beats backwards. Release the
        // sounding gesture and discard every old deadline before any
        // event can reappear on the new timeline.
        if self
            .last_tick_beat
            .map(|previous| now + f64::EPSILON < previous)
            .unwrap_or(false)
        {
            if let Some(gesture) = self.species4.take() {
                if gesture.stage != Species4Stage::Armed {
                    ops.push(DispatchOp::NoteOff {
                        target: self.voice_output,
                        note: gesture.pitch,
                        channel: gesture.owner_channel,
                    });
                    if let Some(bass) = gesture.bass_pitch {
                        ops.push(DispatchOp::NoteOff {
                            target: self.voice_output,
                            note: bass,
                            channel: gesture.owner_channel,
                        });
                    }
                }
            }
            self.reset_species4_phrase();
        }
        self.last_tick_beat = Some(now);

        if !self.enabled {
            return LaneOutput {
                ops,
                ..Default::default()
            };
        }

        if self.phrase_aware
            && self.species4_phrase_id.is_some()
            && world.phrase_snapshot().phase == PhrasePhase::Idle
        {
            if let Some(gesture) = self.species4.take() {
                if gesture.stage != Species4Stage::Armed {
                    ops.push(DispatchOp::NoteOff {
                        target: self.voice_output,
                        note: gesture.pitch,
                        channel: gesture.owner_channel,
                    });
                    if let Some(bass) = gesture.bass_pitch {
                        ops.push(DispatchOp::NoteOff {
                            target: self.voice_output,
                            note: bass,
                            channel: gesture.owner_channel,
                        });
                    }
                }
            }
            self.reset_species4_phrase();
        }

        if let Some(mut gesture) = self.species4.take() {
            if gesture.release_by.map(|at| now >= at).unwrap_or(false) {
                if gesture.stage != Species4Stage::Armed {
                    ops.push(DispatchOp::NoteOff {
                        target: self.voice_output,
                        note: gesture.pitch,
                        channel: gesture.owner_channel,
                    });
                    if let Some(bass) = gesture.bass_pitch {
                        ops.push(DispatchOp::NoteOff {
                            target: self.voice_output,
                            note: bass,
                            channel: gesture.owner_channel,
                        });
                    }
                }
            } else {
                if let Species4Stage::Suspended { bass_target, .. } = gesture.stage {
                    if self.phrase_aware && gesture.bass_pitch != Some(bass_target) {
                        if let Some(bass) = gesture.bass_pitch {
                            ops.push(DispatchOp::NoteOff {
                                target: self.voice_output,
                                note: bass,
                                channel: gesture.owner_channel,
                            });
                        }
                        ops.push(DispatchOp::NoteOn {
                            target: self.voice_output,
                            note: bass_target,
                            velocity: gesture.velocity.saturating_sub(24).max(42),
                            channel: gesture.owner_channel,
                        });
                        gesture.bass_pitch = Some(bass_target);
                    }
                }
                match gesture.stage {
                    Species4Stage::Armed if now >= gesture.prepare_at => {
                        if now <= gesture.prepare_at + SPECIES4_LATE_PREPARATION {
                            if let Some(bass) = gesture.bass_pitch {
                                ops.push(DispatchOp::NoteOn {
                                    target: self.voice_output,
                                    note: bass,
                                    velocity: gesture.velocity.saturating_sub(24).max(42),
                                    channel: gesture.owner_channel,
                                });
                            }
                            ops.push(DispatchOp::NoteOn {
                                target: self.voice_output,
                                note: gesture.pitch,
                                velocity: gesture.velocity,
                                channel: gesture.owner_channel,
                            });
                            gesture.stage = Species4Stage::Prepared;
                            self.species4 = Some(gesture);
                        }
                    }
                    Species4Stage::Prepared
                        if !self.phrase_aware && now >= gesture.strong_at + 0.5 =>
                    {
                        // Legacy Species IV remains grid-strict: without a
                        // new cantus near the expected strong beat, end the
                        // consonant syncopation instead of inventing a suspension.
                        ops.push(DispatchOp::NoteOff {
                            target: self.voice_output,
                            note: gesture.pitch,
                            channel: gesture.owner_channel,
                        });
                    }
                    Species4Stage::Suspended {
                        resolution,
                        resolve_at,
                        ..
                    } if now >= resolve_at => {
                        ops.push(DispatchOp::NoteOff {
                            target: self.voice_output,
                            note: gesture.pitch,
                            channel: gesture.owner_channel,
                        });
                        ops.push(DispatchOp::NoteOn {
                            target: self.voice_output,
                            note: resolution,
                            velocity: gesture.velocity.saturating_sub(8).max(48),
                            channel: gesture.owner_channel,
                        });
                        gesture.pitch = resolution;
                        gesture.prepare_at = resolve_at;
                        gesture.strong_at = resolve_at + 0.5;
                        gesture.stage = if self.phrase_aware {
                            Species4Stage::Resolved {
                                release_at: resolve_at + 0.5,
                            }
                        } else {
                            Species4Stage::Prepared
                        };
                        self.species4 = Some(gesture);
                    }
                    Species4Stage::Resolved { release_at } if now >= release_at => {
                        ops.push(DispatchOp::NoteOff {
                            target: self.voice_output,
                            note: gesture.pitch,
                            channel: gesture.owner_channel,
                        });
                        if let Some(bass) = gesture.bass_pitch {
                            ops.push(DispatchOp::NoteOff {
                                target: self.voice_output,
                                note: bass,
                                channel: gesture.owner_channel,
                            });
                        }
                        self.species4_completed = self.species4_completed.saturating_add(1);
                        self.species4_breathe_until = Some(release_at + 1.0);
                    }
                    _ => self.species4 = Some(gesture),
                }
            }
        }

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

        LaneOutput {
            ops,
            ..Default::default()
        }
    }

    fn tick_slotted(&mut self, world: &WorldState) -> Vec<(u8, DispatchOp)> {
        let bass_before = self.species4.and_then(|gesture| gesture.bass_pitch);
        let cleanup_before = self.species4_bass_cleanup.map(|(note, _)| note);
        let output = self.tick(world);
        let bass_after = self.species4.and_then(|gesture| gesture.bass_pitch);
        output
            .ops
            .into_iter()
            .map(|op| {
                let note = match op {
                    DispatchOp::NoteOn { note, .. } | DispatchOp::NoteOff { note, .. } => note,
                    _ => return (0, op),
                };
                let bass = [cleanup_before, bass_before, bass_after]
                    .into_iter()
                    .flatten()
                    .any(|candidate| candidate == note);
                (u8::from(bass), op)
            })
            .collect()
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
            "phrase_aware": self.phrase_aware,
            "hold_mode": self.hold_mode.map(hold_mode_to_json),
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
        if let Some(phrase_aware) = state.get("phrase_aware").and_then(|v| v.as_bool()) {
            if phrase_aware != self.phrase_aware {
                self.flush_species4();
                self.reset_species4_phrase();
                self.phrase_aware = phrase_aware;
            }
        }
        // Lane-level HoldMode override. Absent / unknown shape leaves
        // existing mode untouched; explicit JSON null clears to None
        // (inherit global).
        if let Some(v) = state.get("hold_mode") {
            if v.is_null() {
                self.hold_mode = None;
            } else if let Some(hm) = hold_mode_from_json(v) {
                self.hold_mode = Some(hm);
            }
        }
        Ok(())
    }
}

// Suppress unused-field warnings on HarmonyMode (referenced via the
// scale module but not directly used by this file's symbols today).
const _: Option<HarmonyMode> = None;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::WorldState;
    use contrapunk_harmony::HarmonyEngine;
    use contrapunk_transport::Transport;
    use std::sync::Arc;

    fn fixture() -> (CounterpointLane, Arc<WorldState>, Arc<Transport>) {
        let transport = Transport::new(48_000);
        let engine = Arc::new(std::sync::Mutex::new(HarmonyEngine::new(
            contrapunk_harmony::Key::C,
            HarmonyMode::PassThrough,
        )));
        let world = WorldState::new(Arc::clone(&transport), engine);
        let lane = CounterpointLane::new();
        (lane, world, transport)
    }

    fn advance_to_beat(transport: &Transport, beat: f64) {
        let spb = transport.sample_rate() as f64 * 60.0 / transport.bpm();
        let target_frames = (beat * spb) as u32;
        transport.reset();
        transport.play();
        if target_frames > 0 {
            let _ = transport.advance(target_frames);
        }
    }

    // HoldMode tests (#11) — CounterpointLane

    #[test]
    fn hold_mode_cancel_drops_pending_species2() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_species(CounterpointSpecies::Species2);
        *world.global_hold_mode.lock().unwrap() = HoldMode::Cancel;

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        assert!(
            !lane.pending_on.is_empty(),
            "Species 2 should buffer at least one pending emission"
        );

        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        assert!(
            lane.pending_on.iter().all(|p| p.player_note != 60),
            "Cancel should drop pending for note 60"
        );
    }

    #[test]
    fn hold_mode_near_future_keeps_within_window() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_species(CounterpointSpecies::Species2);
        *world.global_hold_mode.lock().unwrap() = HoldMode::NearFuture { tail_beats: 0.6 };

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        assert!(!lane.pending_on.is_empty());

        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        let kept = lane
            .pending_on
            .iter()
            .filter(|p| p.player_note == 60)
            .count();
        assert!(
            kept > 0,
            "NearFuture(0.6) should keep the 0.5b pending entry"
        );
    }

    #[test]
    fn hold_mode_near_future_drops_unsounded_species4_gesture() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_species(CounterpointSpecies::Species4);
        *world.global_hold_mode.lock().unwrap() = HoldMode::NearFuture { tail_beats: 0.1 };

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        assert_eq!(lane.species4.map(|g| g.stage), Some(Species4Stage::Armed));

        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.1);
        assert!(lane.tick(&world).ops.is_empty());
        assert!(lane.species4.is_none());
    }

    #[test]
    fn species4_emits_preparation_hold_and_downward_resolution() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);
        lane.transpose_degrees = 2;
        lane.hold_mode = Some(HoldMode::NearFuture { tail_beats: 2.0 });

        // C on the strong beat prepares E on the following weak half.
        advance_to_beat(&transport, 0.0);
        assert!(lane
            .on_input(
                InputEvent::NoteOn {
                    note: 60,
                    velocity: 100,
                    channel: 0,
                },
                &world,
            )
            .ops
            .is_empty());
        assert!(lane.tick(&world).ops.is_empty());

        advance_to_beat(&transport, 0.5);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 64,
                velocity: 100,
                channel: 0,
            }]
        );

        // F makes the retained E a valid 4-3-style dissonance. The
        // old legato NoteOff must not terminate the transferred cycle.
        advance_to_beat(&transport, 1.0);
        assert!(lane
            .on_input(
                InputEvent::NoteOn {
                    note: 65,
                    velocity: 96,
                    channel: 0,
                },
                &world,
            )
            .ops
            .is_empty());
        assert!(lane
            .on_input(
                InputEvent::NoteOff {
                    note: 60,
                    channel: 0,
                },
                &world,
            )
            .ops
            .is_empty());
        assert!(
            lane.tick(&world).ops.is_empty(),
            "strong beat must not retrigger"
        );

        advance_to_beat(&transport, 1.5);
        assert_eq!(
            lane.tick(&world).ops,
            vec![
                DispatchOp::NoteOff {
                    target: VoiceOutputTarget::Synth,
                    note: 64,
                    channel: 0,
                },
                DispatchOp::NoteOn {
                    target: VoiceOutputTarget::Synth,
                    note: 62,
                    velocity: 88,
                    channel: 0,
                },
            ]
        );

        // With no next strong-beat cantus, the consonant resolution
        // releases on the following weak boundary.
        advance_to_beat(&transport, 2.5);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOff {
                target: VoiceOutputTarget::Synth,
                note: 62,
                channel: 0,
            }]
        );
        assert!(lane.species4.is_none());
    }

    #[test]
    fn species4_below_voice_resolves_in_the_players_current_scale() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);
        lane.transpose_degrees = -2;
        lane.prefer_above = false;
        lane.hold_mode = Some(HoldMode::NearFuture { tail_beats: 2.0 });
        assert_eq!(
            world.engine_snapshot.lock().unwrap().scale_mode(),
            ScaleMode::Ionian
        );

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.5);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOn {
                target: VoiceOutputTarget::Synth,
                note: 57,
                velocity: 100,
                channel: 0,
            }]
        );

        advance_to_beat(&transport, 1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 59,
                velocity: 96,
                channel: 0,
            },
            &world,
        );
        assert!(lane
            .on_input(
                InputEvent::NoteOff {
                    note: 60,
                    channel: 0,
                },
                &world,
            )
            .ops
            .is_empty());

        advance_to_beat(&transport, 1.5);
        assert_eq!(
            lane.tick(&world).ops,
            vec![
                DispatchOp::NoteOff {
                    target: VoiceOutputTarget::Synth,
                    note: 57,
                    channel: 0,
                },
                DispatchOp::NoteOn {
                    target: VoiceOutputTarget::Synth,
                    note: 55,
                    velocity: 88,
                    channel: 0,
                },
            ]
        );
    }

    #[test]
    fn phrase_aware_species4_resolves_unquantized_attacks() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);
        lane.transpose_degrees = -2;
        lane.prefer_above = false;
        lane.phrase_aware = true;

        advance_to_beat(&transport, 0.0);
        let opening = InputEvent::NoteOn {
            note: 67,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(opening);
        lane.on_input(opening, &world);

        advance_to_beat(&transport, 0.7);
        let preparation = InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(preparation);
        lane.on_input(preparation, &world);
        advance_to_beat(&transport, 1.5);
        assert_eq!(
            lane.tick_slotted(&world),
            vec![
                (
                    1,
                    DispatchOp::NoteOn {
                        target: VoiceOutputTarget::Synth,
                        note: 41,
                        velocity: 76,
                        channel: 0,
                    },
                ),
                (
                    0,
                    DispatchOp::NoteOn {
                        target: VoiceOutputTarget::Synth,
                        note: 57,
                        velocity: 100,
                        channel: 0,
                    },
                ),
            ]
        );

        // 2.23 is deliberately outside the legacy ±0.15-beat window
        // around the expected strong beat at 2.0.
        advance_to_beat(&transport, 2.23);
        let suspension = InputEvent::NoteOn {
            note: 59,
            velocity: 96,
            channel: 0,
        };
        world.observe_phrase_input(suspension);
        lane.on_input(suspension, &world);
        assert_eq!(
            lane.tick_slotted(&world),
            vec![
                (
                    1,
                    DispatchOp::NoteOff {
                        target: VoiceOutputTarget::Synth,
                        note: 41,
                        channel: 0,
                    },
                ),
                (
                    1,
                    DispatchOp::NoteOn {
                        target: VoiceOutputTarget::Synth,
                        note: 40,
                        velocity: 72,
                        channel: 0,
                    },
                ),
            ]
        );
        advance_to_beat(&transport, 2.73);
        assert_eq!(
            lane.tick_slotted(&world),
            vec![
                (
                    0,
                    DispatchOp::NoteOff {
                        target: VoiceOutputTarget::Synth,
                        note: 57,
                        channel: 0,
                    },
                ),
                (
                    0,
                    DispatchOp::NoteOn {
                        target: VoiceOutputTarget::Synth,
                        note: 55,
                        velocity: 88,
                        channel: 0,
                    },
                ),
            ]
        );
    }

    #[test]
    fn phrase_aware_species4_exposes_opening_then_resolves_and_breathes() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);
        lane.transpose_degrees = -2;
        lane.prefer_above = false;
        lane.phrase_aware = true;
        lane.hold_mode = Some(HoldMode::NearFuture { tail_beats: 2.0 });

        advance_to_beat(&transport, 0.0);
        let opening = InputEvent::NoteOn {
            note: 67,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(opening);
        assert!(lane.on_input(opening, &world).ops.is_empty());
        assert!(
            lane.species4.is_none(),
            "the phrase opening must remain exposed"
        );

        advance_to_beat(&transport, 0.25);
        let release = InputEvent::NoteOff {
            note: 67,
            channel: 0,
        };
        world.observe_phrase_input(release);
        lane.on_input(release, &world);

        advance_to_beat(&transport, 1.0);
        let preparation_source = InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(preparation_source);
        lane.on_input(preparation_source, &world);
        advance_to_beat(&transport, 1.5);
        assert_eq!(
            lane.tick(&world).ops,
            vec![
                DispatchOp::NoteOn {
                    target: VoiceOutputTarget::Synth,
                    note: 41,
                    velocity: 76,
                    channel: 0,
                },
                DispatchOp::NoteOn {
                    target: VoiceOutputTarget::Synth,
                    note: 57,
                    velocity: 100,
                    channel: 0,
                },
            ]
        );

        advance_to_beat(&transport, 2.0);
        let suspension_source = InputEvent::NoteOn {
            note: 59,
            velocity: 96,
            channel: 0,
        };
        world.observe_phrase_input(suspension_source);
        assert!(lane.on_input(suspension_source, &world).ops.is_empty());
        let old_release = InputEvent::NoteOff {
            note: 60,
            channel: 0,
        };
        world.observe_phrase_input(old_release);
        lane.on_input(old_release, &world);
        assert_eq!(
            lane.tick(&world).ops,
            vec![
                DispatchOp::NoteOff {
                    target: VoiceOutputTarget::Synth,
                    note: 41,
                    channel: 0,
                },
                DispatchOp::NoteOn {
                    target: VoiceOutputTarget::Synth,
                    note: 40,
                    velocity: 72,
                    channel: 0,
                },
            ]
        );

        advance_to_beat(&transport, 2.5);
        assert_eq!(
            lane.tick(&world).ops,
            vec![
                DispatchOp::NoteOff {
                    target: VoiceOutputTarget::Synth,
                    note: 57,
                    channel: 0,
                },
                DispatchOp::NoteOn {
                    target: VoiceOutputTarget::Synth,
                    note: 55,
                    velocity: 88,
                    channel: 0,
                },
            ]
        );
        advance_to_beat(&transport, 3.0);
        assert_eq!(
            lane.tick(&world).ops,
            vec![
                DispatchOp::NoteOff {
                    target: VoiceOutputTarget::Synth,
                    note: 55,
                    channel: 0,
                },
                DispatchOp::NoteOff {
                    target: VoiceOutputTarget::Synth,
                    note: 40,
                    channel: 0,
                },
            ]
        );

        advance_to_beat(&transport, 3.25);
        let during_breath = InputEvent::NoteOn {
            note: 62,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(during_breath);
        assert!(lane.on_input(during_breath, &world).ops.is_empty());
        assert!(
            lane.species4.is_none(),
            "the post-resolution breath must stay silent"
        );

        advance_to_beat(&transport, 4.0);
        let second_preparation = InputEvent::NoteOn {
            note: 64,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(second_preparation);
        lane.on_input(second_preparation, &world);
        advance_to_beat(&transport, 4.5);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [
                DispatchOp::NoteOn { note: 45, .. },
                DispatchOp::NoteOn { note: 60, .. }
            ]
        ));

        advance_to_beat(&transport, 5.0);
        let second_suspension = InputEvent::NoteOn {
            note: 62,
            velocity: 96,
            channel: 0,
        };
        world.observe_phrase_input(second_suspension);
        lane.on_input(second_suspension, &world);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [
                DispatchOp::NoteOff { note: 45, .. },
                DispatchOp::NoteOn { note: 43, .. }
            ]
        ));
        advance_to_beat(&transport, 5.5);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [
                DispatchOp::NoteOff { note: 60, .. },
                DispatchOp::NoteOn { note: 59, .. }
            ]
        ));
        advance_to_beat(&transport, 6.0);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [
                DispatchOp::NoteOff { note: 59, .. },
                DispatchOp::NoteOff { note: 43, .. }
            ]
        ));

        advance_to_beat(&transport, 6.25);
        let after_garland = InputEvent::NoteOn {
            note: 67,
            velocity: 100,
            channel: 0,
        };
        world.observe_phrase_input(after_garland);
        assert!(lane.on_input(after_garland, &world).ops.is_empty());
        assert!(
            lane.species4.is_none(),
            "two resolutions are the phrase ceiling"
        );
    }

    #[test]
    fn species4_sounded_note_releases_at_hold_deadline() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);
        lane.transpose_degrees = 2;
        lane.hold_mode = Some(HoldMode::NearFuture { tail_beats: 0.1 });

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.5);
        let _ = lane.tick(&world);
        advance_to_beat(&transport, 0.6);
        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );

        advance_to_beat(&transport, 0.7);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOff {
                target: VoiceOutputTarget::Synth,
                note: 64,
                channel: 0,
            }]
        );
        assert!(lane.species4.is_none());
    }

    #[test]
    fn species4_early_melody_change_releases_before_rearming() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);
        lane.transpose_degrees = 2;

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.5);
        assert!(matches!(
            lane.tick(&world).ops.as_slice(),
            [DispatchOp::NoteOn { note: 64, .. }]
        ));

        advance_to_beat(&transport, 0.75);
        assert_eq!(
            lane.on_input(
                InputEvent::NoteOn {
                    note: 62,
                    velocity: 100,
                    channel: 0,
                },
                &world,
            )
            .ops,
            vec![DispatchOp::NoteOff {
                target: VoiceOutputTarget::Synth,
                note: 64,
                channel: 0,
            }]
        );
        assert_eq!(lane.species4.map(|g| g.stage), Some(Species4Stage::Armed));
    }

    #[test]
    fn species4_disable_flushes_sounded_note_and_cancels_future_events() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);
        lane.transpose_degrees = 2;

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.5);
        let _ = lane.tick(&world);
        advance_to_beat(&transport, 1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 65,
                velocity: 96,
                channel: 0,
            },
            &world,
        );
        assert!(matches!(
            lane.species4.map(|g| g.stage),
            Some(Species4Stage::Suspended { .. })
        ));

        lane.set_enabled(false);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOff {
                target: VoiceOutputTarget::Synth,
                note: 64,
                channel: 0,
            }]
        );
        advance_to_beat(&transport, 2.0);
        assert!(lane.tick(&world).ops.is_empty());
    }

    #[test]
    fn species4_reset_cancels_every_future_phase() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.5);
        let _ = lane.tick(&world);
        advance_to_beat(&transport, 1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 65,
                velocity: 96,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let resolved = lane.tick(&world);
        assert!(matches!(
            resolved.ops.as_slice(),
            [
                DispatchOp::NoteOff { note: 64, .. },
                DispatchOp::NoteOn { note: 62, .. }
            ]
        ));

        // Panic/Stop dispatches All Notes Off before this reset call.
        lane.reset_runtime();
        advance_to_beat(&transport, 2.5);
        assert!(lane.tick(&world).ops.is_empty());
        assert!(lane.species4.is_none());
    }

    #[test]
    fn species4_transport_rewind_releases_active_gesture() {
        let (mut lane, world, transport) = fixture();
        lane.set_species(CounterpointSpecies::Species4);
        lane.transpose_degrees = 2;

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 0.5);
        let _ = lane.tick(&world);

        advance_to_beat(&transport, 0.25);
        assert_eq!(
            lane.tick(&world).ops,
            vec![DispatchOp::NoteOff {
                target: VoiceOutputTarget::Synth,
                note: 64,
                channel: 0,
            }]
        );
        assert!(lane.species4.is_none());
    }

    #[test]
    fn hold_mode_forever_keeps_all_pending() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_species(CounterpointSpecies::Species2);
        *world.global_hold_mode.lock().unwrap() = HoldMode::Forever;

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        let before = lane
            .pending_on
            .iter()
            .filter(|p| p.player_note == 60)
            .count();
        assert!(before > 0);

        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        let after = lane
            .pending_on
            .iter()
            .filter(|p| p.player_note == 60)
            .count();
        assert_eq!(before, after, "Forever should preserve all pending");
    }

    #[test]
    fn hold_mode_lane_override_beats_global() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_species(CounterpointSpecies::Species2);
        *world.global_hold_mode.lock().unwrap() = HoldMode::Cancel;
        lane.hold_mode = Some(HoldMode::Forever);

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        let before = lane
            .pending_on
            .iter()
            .filter(|p| p.player_note == 60)
            .count();
        assert!(before > 0);

        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        let after = lane
            .pending_on
            .iter()
            .filter(|p| p.player_note == 60)
            .count();
        assert_eq!(before, after, "Lane Forever should beat global Cancel");
    }

    #[test]
    fn hold_mode_only_affects_target_player_note() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_species(CounterpointSpecies::Species2);
        *world.global_hold_mode.lock().unwrap() = HoldMode::Cancel;

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        lane.on_input(
            InputEvent::NoteOn {
                note: 64,
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );

        assert!(
            lane.pending_on.iter().all(|p| p.player_note != 60),
            "Note 60's pending should be cancelled"
        );
        assert!(
            lane.pending_on.iter().any(|p| p.player_note == 64),
            "Note 64's pending should be preserved"
        );
    }
}
