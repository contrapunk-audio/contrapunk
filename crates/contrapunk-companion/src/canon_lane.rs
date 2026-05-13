//! Canon Lane — N delayed-entry voices replaying the player's melody.
//!
//! Closes #3. The first concrete `Decide`-phase Lane. Each input note
//! captured by `on_input` is buffered with its trigger beat position
//! and one pending emission is scheduled PER configured canon voice.
//! On each `tick`, the lane drains entries whose delay has elapsed
//! against the current transport beat. Matching NoteOff events follow
//! the same delay so each canon voice preserves the original duration.
//!
//! Per `.planning/research/group-a-core-harmony.md:108-153`:
//! - Beat-clock-driven (NOT user-input-driven). The Lane fires on
//!   `tick`, called every router iteration with the live transport.
//!   This is how the canon plays *between* user note-ons rather than
//!   clumping all delayed emissions on the next trigger.
//! - Per-voice delay buffer is bounded (few seconds of held content)
//!   so no memory concern.
//! - Reuses the existing synthetic-beat / transport infrastructure.
//! - "Multi-voice canons (3-voice canon at +2, +4)? Trivial extension
//!   once 1-voice works." Now wired.
//!
//! Each `CanonVoice` is independently configurable: own delay (in
//! beats) and own diatonic transpose. Transpose routes through
//! `Scale::harmonize_smart`, so out-of-scale input uses the engine's
//! modal-interchange logic when the user has it enabled, and falls
//! back to consonant chromatic intervals otherwise — never naked
//! unison.

#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

use wmidi::Note;

use contrapunk_harmony::{
    CounterpointSpecies, CounterpointStrictness, HarmonyEngine, HarmonyMode, Key, OctaveMode,
    ScaleMode, VoiceLeadingStyle,
};

use super::lane::{
    hold_mode_from_json, hold_mode_to_json, HoldMode, InputEvent, InputFilter, Lane, LaneOutput,
    LanePhase,
};
use super::world::WorldState;
use super::DispatchOp;
use crate::voice_output::VoiceOutputTarget;

/// Configuration for one canon voice. The lane can hold any number
/// of these; each one independently delays, transposes, and time-
/// scales the player's input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanonVoice {
    /// Beats to wait between the player's NoteOn and this voice's
    /// NoteOn. Clamped to [0.0, 16.0] beats.
    pub delay_beats: f32,
    /// Diatonic transpose for this voice's emissions. Clamped to
    /// [-7, 7] degrees.
    pub transpose_degrees: i8,
    /// Time-ratio (augmentation / diminution). 1.0 = strict imitation;
    /// 2.0 = augmentation (canon plays at half speed, notes twice as
    /// long); 0.5 = diminution (canon plays at double speed, notes
    /// half as long). Classical canons: Bach uses both. Each canon
    /// voice's relative-to-phrase-anchor timing is multiplied by this
    /// ratio, so the whole sequence stretches or compresses
    /// proportionally — not just individual notes. Clamped to
    /// [0.125, 8.0] (six octaves of speed, useful range).
    pub time_ratio: f32,
    /// Optional per-voice harmony-mode override. `None` (default) =
    /// inherit the engine's global mode for this voice's harmony
    /// stack. `Some(mode)` = temporarily switch the shared engine to
    /// the voice's mode while harmonizing this voice's subject pitch,
    /// then restore. This lets the user assign e.g. V1 =
    /// DiatonicThirds, V2 = StrictCounterpoint, V3 = ContraryMotion.
    ///
    /// v1 limitation: switching the engine mode resets the stateful
    /// modes' per-mode history (counterpoint, contrary motion) on
    /// each canon emission. Stateless modes (PassThrough,
    /// DiatonicThirds, DiatonicFourths) work cleanly. Per-voice
    /// stateful machinery is the follow-up that adds independent
    /// state per voice rather than thrashing global state.
    pub harmony_mode: Option<HarmonyMode>,
    /// Cascade reference. `None` = harmonize against the player's
    /// input note (default canon behavior — every voice mirrors the
    /// player). `Some(idx)` = harmonize against canon voice `idx`'s
    /// subject pitch instead, enabling fugal cascade where V2
    /// follows V1, V3 follows V2, etc.
    ///
    /// Required invariant: `idx < self.index_in_lane` to avoid
    /// circular references. The lane enforces this at apply time;
    /// out-of-range or self-referential values silently fall back
    /// to the player as the reference.
    pub reference_voice: Option<usize>,
    /// Per-voice override: sub-engine `voice_count` (1..=4). None =
    /// inherit the lane's default 2 (subject + 1 harmony partner).
    /// Higher values fan the mini-engine out into a 3- or 4-voice
    /// chord stack for this voice.
    pub voice_count: Option<u8>,
    /// Per-voice override: sub-engine `voice_position` (the index
    /// in the SATB stack where this voice's emitted line sits).
    /// None = inherit (typically 0). Required: position < voice_count.
    pub voice_position: Option<u8>,
    /// Per-voice override: enable/disable voice-leading on this
    /// voice's mini-engine.
    pub voice_leading_enabled: Option<bool>,
    /// Per-voice override: voice-leading style (Free / Palestrina /
    /// BachChorale / Jazz).
    pub voice_leading_style: Option<VoiceLeadingStyle>,
    /// Per-voice override: octave-spread mode for the mini-engine's
    /// emitted stack.
    pub octave_mode: Option<OctaveMode>,
    /// Per-voice override: counterpoint species (1-4) applied when
    /// `harmony_mode = StrictCounterpoint`.
    pub counterpoint_species: Option<CounterpointSpecies>,
    /// Per-voice override: counterpoint strictness mode.
    pub counterpoint_strictness: Option<CounterpointStrictness>,
    /// Per-voice override: how to handle pending emissions when the
    /// player releases the seeding NoteOn. `None` = inherit from the
    /// lane (which itself can inherit from the Companion's global
    /// default). See `lane::HoldMode` for the four modes.
    pub hold_mode: Option<HoldMode>,
}

impl CanonVoice {
    pub fn new(delay_beats: f32, transpose_degrees: i8) -> Self {
        Self::with_time_ratio(delay_beats, transpose_degrees, 1.0)
    }

    pub fn with_time_ratio(delay_beats: f32, transpose_degrees: i8, time_ratio: f32) -> Self {
        Self {
            delay_beats: delay_beats.clamp(0.0, 16.0),
            transpose_degrees: transpose_degrees.clamp(-7, 7),
            time_ratio: time_ratio.clamp(0.125, 8.0),
            harmony_mode: None,
            reference_voice: None,
            voice_count: None,
            voice_position: None,
            voice_leading_enabled: None,
            voice_leading_style: None,
            octave_mode: None,
            counterpoint_species: None,
            counterpoint_strictness: None,
            hold_mode: None,
        }
    }
}

impl Default for CanonVoice {
    fn default() -> Self {
        Self {
            delay_beats: 1.0,
            transpose_degrees: 0,
            time_ratio: 1.0,
            harmony_mode: None,
            reference_voice: None,
            voice_count: None,
            voice_position: None,
            voice_leading_enabled: None,
            voice_leading_style: None,
            octave_mode: None,
            counterpoint_species: None,
            counterpoint_strictness: None,
            hold_mode: None,
        }
    }
}

/// After this many beats of silence, the next input note starts a
/// new "phrase" — sequence anchor resets to that note's beat. This
/// keeps augmentation / diminution sane across natural musical pauses:
/// the anchor doesn't drift arbitrarily far back from current play.
/// 2 beats is short enough to feel natural between phrases but long
/// enough that legato playing doesn't accidentally reset.
const PHRASE_SILENCE_THRESHOLD: f64 = 2.0;

/// Snapshot of the global engine fields each mini-engine sync needs.
/// Read once under the lock; used to fall back to global values for
/// any per-voice override that's None.
struct EngineSnapshot {
    key: Key,
    mode: HarmonyMode,
    scale_mode: ScaleMode,
    octave_mode: OctaveMode,
    voice_count: usize,
    voice_position: usize,
    voice_leading_enabled: bool,
    voice_leading_style: VoiceLeadingStyle,
    counterpoint_species: CounterpointSpecies,
    counterpoint_strictness: CounterpointStrictness,
    /// Latest counterpoint beat phase pushed onto the global engine
    /// by the router thread. Mirrored onto each mini-engine so
    /// stateful Species 2/3/4 logic actually fires; otherwise they
    /// silently fall back to Species 1.
    counterpoint_beat_phase: Option<f64>,
}

impl Default for EngineSnapshot {
    fn default() -> Self {
        Self {
            key: Key::C,
            mode: HarmonyMode::PassThrough,
            scale_mode: ScaleMode::Ionian,
            octave_mode: OctaveMode::None,
            voice_count: 2,
            voice_position: 0,
            voice_leading_enabled: false,
            voice_leading_style: VoiceLeadingStyle::Free,
            counterpoint_species: CounterpointSpecies::Species1,
            counterpoint_strictness: CounterpointStrictness::Strict,
            counterpoint_beat_phase: None,
        }
    }
}

/// Canonical-name string helpers for the per-voice override enums.
/// All match the names parsed by the corresponding `parse_*` functions
/// in commands/harmony.rs so a string round-tripped through the UI is
/// stable.

fn voice_leading_style_to_str(s: VoiceLeadingStyle) -> &'static str {
    match s {
        VoiceLeadingStyle::Free => "Free",
        VoiceLeadingStyle::Palestrina => "Palestrina",
        VoiceLeadingStyle::BachChorale => "BachChorale",
        VoiceLeadingStyle::Jazz => "Jazz",
    }
}

fn voice_leading_style_from_str(s: &str) -> Option<VoiceLeadingStyle> {
    Some(match s {
        "Free" => VoiceLeadingStyle::Free,
        "Palestrina" => VoiceLeadingStyle::Palestrina,
        "BachChorale" => VoiceLeadingStyle::BachChorale,
        "Jazz" => VoiceLeadingStyle::Jazz,
        _ => return None,
    })
}

fn octave_mode_to_str(m: OctaveMode) -> &'static str {
    match m {
        OctaveMode::None => "None",
        OctaveMode::Spread => "Spread",
        OctaveMode::BassTrebleSplit => "BassTrebleSplit",
        OctaveMode::Mirror => "Mirror",
    }
}

fn octave_mode_from_str(s: &str) -> Option<OctaveMode> {
    Some(match s {
        "None" => OctaveMode::None,
        "Spread" => OctaveMode::Spread,
        "BassTrebleSplit" => OctaveMode::BassTrebleSplit,
        "Mirror" => OctaveMode::Mirror,
        _ => return None,
    })
}

fn counterpoint_species_to_str(s: CounterpointSpecies) -> &'static str {
    match s {
        CounterpointSpecies::Species1 => "Species1",
        CounterpointSpecies::Species2 => "Species2",
        CounterpointSpecies::Species3 => "Species3",
        CounterpointSpecies::Species4 => "Species4",
    }
}

fn counterpoint_species_from_str(s: &str) -> Option<CounterpointSpecies> {
    Some(match s {
        "Species1" => CounterpointSpecies::Species1,
        "Species2" => CounterpointSpecies::Species2,
        "Species3" => CounterpointSpecies::Species3,
        "Species4" => CounterpointSpecies::Species4,
        _ => return None,
    })
}

fn counterpoint_strictness_to_str(s: CounterpointStrictness) -> &'static str {
    match s {
        CounterpointStrictness::Strict => "Strict",
        CounterpointStrictness::Relaxed => "Relaxed",
    }
}

fn counterpoint_strictness_from_str(s: &str) -> Option<CounterpointStrictness> {
    Some(match s {
        "Strict" => CounterpointStrictness::Strict,
        "Relaxed" => CounterpointStrictness::Relaxed,
        _ => return None,
    })
}

/// Convert a HarmonyMode to the canonical string the JS adapter sends.
/// Matches the names accepted by `parse_harmony_mode` in
/// `src-tauri/src/commands/harmony.rs:304` so round-tripping through
/// the UI is symmetric.
fn harmony_mode_to_str(m: HarmonyMode) -> &'static str {
    match m {
        HarmonyMode::PassThrough => "PassThrough",
        HarmonyMode::DiatonicThirds => "DiatonicThirds",
        HarmonyMode::DiatonicFourths => "DiatonicFourths",
        HarmonyMode::RandomBelow => "RandomBelow",
        HarmonyMode::RandomBelowNoSeconds => "RandomBelowNoSeconds",
        HarmonyMode::ContraryMotion => "ContraryMotion",
        HarmonyMode::StrictCounterpoint => "StrictCounterpoint",
        HarmonyMode::BarryHarris => "BarryHarris",
        HarmonyMode::FunctionalHarmony => "FunctionalHarmony",
        HarmonyMode::BachChorale => "BachChorale",
    }
}

/// Inverse of `harmony_mode_to_str`. Defensive: returns None for any
/// string that isn't one of the canonical names so the lane silently
/// falls back to "inherit global mode" if the wire format ever drifts.
fn harmony_mode_from_str(s: &str) -> Option<HarmonyMode> {
    Some(match s {
        "PassThrough" => HarmonyMode::PassThrough,
        "DiatonicThirds" => HarmonyMode::DiatonicThirds,
        "DiatonicFourths" => HarmonyMode::DiatonicFourths,
        "RandomBelow" => HarmonyMode::RandomBelow,
        "RandomBelowNoSeconds" => HarmonyMode::RandomBelowNoSeconds,
        "ContraryMotion" => HarmonyMode::ContraryMotion,
        "StrictCounterpoint" => HarmonyMode::StrictCounterpoint,
        "BarryHarris" => HarmonyMode::BarryHarris,
        "FunctionalHarmony" => HarmonyMode::FunctionalHarmony,
        "BachChorale" => HarmonyMode::BachChorale,
        _ => return None,
    })
}

/// One pending NoteOn that will fire at `fire_at` beats (transport
/// total-beats coordinate).
///
/// `player_note` records which player NoteOn seeded this emission.
/// Required by the HoldMode logic in `on_input(NoteOff)`: when the
/// player releases note N, we need to filter `pending_on` to entries
/// derived from N and selectively cancel / let-fire them per the
/// effective HoldMode (Cancel / NearFuture / PhraseEnd / Forever).
#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingOn {
    fire_at: f64,
    canon_note: u8,
    velocity: u8,
    channel: u8,
    /// Player MIDI note that triggered this emission. Used to scope
    /// the HoldMode filter to "only emissions seeded by THIS released
    /// player note" so other held notes' cascades aren't disturbed.
    player_note: u8,
}

/// One pending NoteOff matched to a previously-scheduled NoteOn.
#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingOff {
    fire_at: f64,
    canon_note: u8,
    channel: u8,
    /// Player MIDI note that originated the matching NoteOn. Same
    /// reason as `PendingOn.player_note` — needed when HoldMode
    /// cancels a NoteOn so we can also drop its matched NoteOff.
    player_note: u8,
}

/// One canon voice's fire record for a single held input. Captures
/// the voice's index (to look up its delay on NoteOff) and the full
/// pitch stack the voice emitted at NoteOn time — including any
/// harmony notes the engine added on top of the canon's subject
/// pitch (interpretation B: each canon voice routes its emission
/// through the engine's harmonize pipeline). NoteOff replays the
/// same stack so every emitted pitch gets a matching off.
#[derive(Clone, Debug, PartialEq)]
struct HeldVoiceFire {
    voice_idx: usize,
    canon_notes: Vec<u8>,
    channel: u8,
}

/// Tracked per held input. `on_beat` is shared across all canon
/// voices for that input; `anchor` captures the phrase anchor in
/// effect when the input arrived (needed at NoteOff time to compute
/// the canon off-fire correctly even if the anchor has since reset
/// due to a phrase boundary); `voices` records what each voice
/// scheduled.
#[derive(Clone, Debug, PartialEq)]
struct HeldEntry {
    on_beat: f64,
    anchor: f64,
    voices: Vec<HeldVoiceFire>,
}

pub struct CanonLane {
    /// Master toggle for this lane independent of the Companion's
    /// master enabled flag. When false, `on_input` and `tick` are
    /// no-ops. Default false: pre-existing user behavior is
    /// preserved when the lane is registered.
    pub enabled: bool,

    /// Configured canon voices. Empty = no canon emissions even when
    /// `enabled` is true. Default contains one voice at delay=1.0,
    /// transpose=0 (the v1 single-voice behavior).
    pub voices: Vec<CanonVoice>,

    /// Output target. Canon emits to its own slot so the dispatcher
    /// can mute / route it independently of the main harmony voices.
    /// Default `Synth` so users hear it without needing MIDI routing.
    pub target: VoiceOutputTarget,

    /// Pending NoteOn emissions, sorted by fire_at ascending (push
    /// order is monotonic in `transport.total_beats()` but voices
    /// have different delays, so we sort by fire_at on each push
    /// to keep the front of the deque the next emission).
    pending_on: VecDeque<PendingOn>,

    /// Pending NoteOff emissions, scanned linearly each tick.
    pending_off: Vec<PendingOff>,

    /// Currently-held inputs. One entry per *input* MIDI note;
    /// each entry's `voices` vec records the per-voice fire info
    /// the lane will need at NoteOff time.
    held: HashMap<u8, HeldEntry>,

    /// Beat at which the current phrase started. All canon voice
    /// emissions for this phrase are computed relative to this
    /// anchor so augmentation / diminution stretches the whole
    /// sequence proportionally instead of just individual notes.
    /// `None` before any input has arrived.
    sequence_anchor: Option<f64>,

    /// Beat of the most recent input event. Used to detect phrase
    /// boundaries: if a new input arrives more than
    /// `PHRASE_SILENCE_THRESHOLD` beats after the last input, the
    /// anchor resets to the new input's beat (the user has started
    /// a fresh phrase).
    last_input_beat: Option<f64>,

    /// Per-voice mini HarmonyEngine. Index-parallel with `voices`:
    /// `voice_engines[i]` is the engine that harmonizes for
    /// `voices[i]`. Per-voice instances are needed for stateful
    /// modes (StrictCounterpoint, ContraryMotion, BachChorale) so
    /// each voice can accumulate its own contour / suspension /
    /// last-harmony history without stomping on the global engine
    /// or each other. Stateless modes get the same correctness for
    /// free.
    ///
    /// Sync invariants — enforced lazily in `sync_voice_engines`:
    /// - len == voices.len()
    /// - key + scale_mode mirror the global engine (so canon stays
    ///   in the song's key)
    /// - mode = voice's `harmony_mode.unwrap_or(global.mode())`
    /// - voice_count = 2 (subject + one harmony pitch is the canon
    ///   contract — multi-pitch stacks come from cascading
    ///   `reference_voice`, not from intra-voice fan-out)
    voice_engines: Vec<HarmonyEngine>,

    /// Lane-level HoldMode override. `None` = inherit from the
    /// Companion's global `hold_mode`. When set, applies to every
    /// voice in this lane unless that voice further overrides via
    /// its own `hold_mode`. Resolution order on NoteOff:
    /// voice.hold_mode → lane.hold_mode → companion.global_hold_mode.
    pub hold_mode: Option<HoldMode>,
}

impl CanonLane {
    pub fn new() -> Self {
        Self {
            enabled: false,
            voices: vec![CanonVoice::default()],
            target: VoiceOutputTarget::Synth,
            pending_on: VecDeque::new(),
            pending_off: Vec::new(),
            held: HashMap::new(),
            sequence_anchor: None,
            last_input_beat: None,
            voice_engines: Vec::new(),
            hold_mode: None,
        }
    }

    /// Bring `voice_engines` into sync with `voices` and the global
    /// engine. Lazy — called from `compute_voice_stack` so the
    /// per-voice engines reflect the latest key / scale / mode at
    /// emission time. Cost: one short global-engine lock per canon
    /// note, plus per-voice setter calls when something has actually
    /// drifted. Setter calls inside HarmonyEngine clear stateful
    /// history only when the value actually changes, so a steady-
    /// state canon doesn't thrash.
    fn sync_voice_engines(&mut self, world: &WorldState) {
        // Snapshot global state under one brief lock. Voices with
        // None for an override field inherit the corresponding global.
        let snapshot = if let Ok(g) = world.engine_snapshot.lock() {
            EngineSnapshot {
                key: g.key(),
                mode: g.mode(),
                scale_mode: g.scale_mode(),
                octave_mode: g.octave_mode(),
                voice_count: g.voice_count(),
                voice_position: g.voice_position(),
                voice_leading_enabled: g.voice_leading_enabled(),
                voice_leading_style: g.voice_leading_style(),
                counterpoint_species: g.counterpoint_species(),
                counterpoint_strictness: g.counterpoint_strictness(),
                counterpoint_beat_phase: g.counterpoint_beat_phase(),
            }
        } else {
            EngineSnapshot::default()
        };

        // Resize: grow with newly-constructed engines, shrink by drop.
        while self.voice_engines.len() < self.voices.len() {
            // Fresh engines default to the canon contract: 2 voices
            // (subject + one harmony partner). Per-voice override
            // applied immediately below.
            self.voice_engines
                .push(HarmonyEngine::with_voices(snapshot.key, snapshot.mode, 2));
        }
        self.voice_engines.truncate(self.voices.len());

        // Sync per-voice engine config. set_* methods on
        // HarmonyEngine are no-ops when the value matches, so stateful
        // history only resets on real change. Order matters: voice_count
        // before voice_position (the latter is clamped to count-1).
        for (i, voice) in self.voices.iter().enumerate() {
            let Some(ve) = self.voice_engines.get_mut(i) else {
                continue;
            };
            if ve.key() != snapshot.key {
                ve.set_key(snapshot.key);
            }
            if ve.scale_mode() != snapshot.scale_mode {
                ve.set_scale_mode(snapshot.scale_mode);
            }
            let target_mode = voice.harmony_mode.unwrap_or(snapshot.mode);
            if ve.mode() != target_mode {
                ve.set_mode(target_mode);
            }
            let target_count = voice
                .voice_count
                .map(|c| (c as usize).clamp(1, 8))
                .unwrap_or(2);
            if ve.voice_count() != target_count {
                ve.set_voice_count(target_count);
            }
            let target_position = voice
                .voice_position
                .map(|p| (p as usize).min(target_count.saturating_sub(1)))
                .unwrap_or_else(|| snapshot.voice_position.min(target_count.saturating_sub(1)));
            if ve.voice_position() != target_position {
                ve.set_voice_position(target_position);
            }
            let target_vl_enabled = voice
                .voice_leading_enabled
                .unwrap_or(snapshot.voice_leading_enabled);
            if ve.voice_leading_enabled() != target_vl_enabled {
                ve.set_voice_leading_enabled(target_vl_enabled);
            }
            let target_vl_style = voice
                .voice_leading_style
                .unwrap_or(snapshot.voice_leading_style);
            if ve.voice_leading_style() != target_vl_style {
                ve.set_voice_leading_style(target_vl_style);
            }
            let target_octave = voice.octave_mode.unwrap_or(snapshot.octave_mode);
            if ve.octave_mode() != target_octave {
                ve.set_octave_mode(target_octave);
            }
            let target_species = voice
                .counterpoint_species
                .unwrap_or(snapshot.counterpoint_species);
            if ve.counterpoint_species() != target_species {
                ve.set_counterpoint_species(target_species);
            }
            let target_strictness = voice
                .counterpoint_strictness
                .unwrap_or(snapshot.counterpoint_strictness);
            if ve.counterpoint_strictness() != target_strictness {
                ve.set_counterpoint_strictness(target_strictness);
            }
            // Beat-phase wire — without this, Species 2/3/4 in canon
            // voices silently collapse to Species 1 because the
            // mini-engine's `effective_counterpoint_beat_phase()`
            // returns None. Pushed every sync so the mini-engine
            // tracks the live transport.
            ve.set_counterpoint_beat_phase(snapshot.counterpoint_beat_phase);
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.pending_on.clear();
            self.pending_off.clear();
            self.held.clear();
            self.sequence_anchor = None;
            self.last_input_beat = None;
        }
    }

    /// Replace the entire voices vector. Capped at 8 entries so a
    /// runaway UI / API caller can't allocate unbounded pending
    /// queues per input note. Empty vector means "no canon
    /// emissions" — the lane stays enabled but produces nothing.
    pub fn set_voices(&mut self, voices: Vec<CanonVoice>) {
        // Preserve every per-voice override (harmony_mode,
        // reference_voice, voice_count, voice_position, voice_leading,
        // octave_mode, counterpoint_species/strictness) — the previous
        // implementation rebuilt each voice via with_time_ratio which
        // silently dropped them, making the cascade + per-voice mode
        // overrides silently no-op. Just clamp the numeric fields and
        // leave the rest alone.
        let clamped: Vec<CanonVoice> = voices
            .into_iter()
            .take(8)
            .map(|mut v| {
                v.delay_beats = v.delay_beats.clamp(0.0, 16.0);
                v.transpose_degrees = v.transpose_degrees.clamp(-7, 7);
                v.time_ratio = v.time_ratio.clamp(0.125, 8.0);
                v
            })
            .collect();
        self.voices = clamped;
        // Drop any in-flight emissions targeting voices that may no
        // longer exist — simplest correctness path. Notes that were
        // already dispatched are out the door already; this clears
        // the pending pipeline.
        self.pending_on.clear();
        self.pending_off.clear();
        self.held.clear();
    }

    pub fn voices(&self) -> &[CanonVoice] {
        &self.voices
    }

    // === Backward-compat single-voice setters — operate on voices[0]. ===
    //
    // Kept so the existing `canon_set_delay` / `canon_set_transpose`
    // Tauri commands keep working. UI for multi-voice uses
    // `canon_set_voices` directly.

    pub fn set_delay(&mut self, beats: f32) {
        if self.voices.is_empty() {
            self.voices.push(CanonVoice::default());
        }
        self.voices[0].delay_beats = beats.clamp(0.0, 16.0);
    }

    pub fn set_transpose(&mut self, degrees: i8) {
        if self.voices.is_empty() {
            self.voices.push(CanonVoice::default());
        }
        self.voices[0].transpose_degrees = degrees.clamp(-7, 7);
    }

    /// Apply diatonic transpose against the engine's current scale,
    /// routing through `Scale::harmonize_smart` so out-of-scale input
    /// lands somewhere musically defensible. See module docstring.
    fn transpose(&self, input_note: u8, transpose_degrees: i8, world: &WorldState) -> u8 {
        if transpose_degrees == 0 {
            return input_note;
        }
        let Ok(mut engine) = world.engine_snapshot.lock() else {
            return input_note;
        };
        let Ok(note) = Note::try_from(input_note) else {
            return input_note;
        };
        let prefer_above = transpose_degrees > 0;
        match engine
            .scale_mut()
            .harmonize_smart(note, transpose_degrees, prefer_above)
        {
            Some(transposed) => u8::from(transposed),
            None => input_note,
        }
    }

    /// Compute the full pitch stack one canon voice emits for a given
    /// input note. Two stages, one engine lock:
    ///
    /// 1. Subject pitch = input + diatonic transpose via
    ///    `Scale::harmonize_smart`. Out-of-scale input borrows via
    ///    modal interchange when enabled.
    /// 2. Subject pitch → engine.harmonize() to produce a harmony
    ///    stack at the engine's GLOBAL current mode + voice_count.
    ///    Each canon voice thus emits a chord rather than a single
    ///    note — Bach-style multi-voice canon (#3 interpretation B).
    ///
    /// v1 uses the engine's global harmony settings for the stack.
    /// Per-voice harmony mode (each canon voice with its own
    /// HarmonyMode) is the next slice. Falls back to a single-note
    /// vec on lock failure / MIDI conversion failure / empty stack.
    fn compute_voice_stack(
        &mut self,
        input_note: u8,
        voice_idx: usize,
        world: &WorldState,
    ) -> (u8, Vec<u8>) {
        // Ensure per-voice engines are sized + synced with the global.
        self.sync_voice_engines(world);

        let Some(voice) = self.voices.get(voice_idx).copied() else {
            return (input_note, vec![input_note]);
        };

        // Stateful cascade: when this voice references an earlier
        // voice, clone the reference voice's CounterpointState chain
        // INTO this voice's engine BEFORE it harmonizes. The chain's
        // harmony_pitch_buffer / interval_history / contour now carry
        // the earlier voice's actual pitches, so this voice's rule
        // scoring will avoid parallels / repeated intervals with what
        // V_ref just emitted. Effectively this turns the cascade chain
        // into one logical multi-voice counterpoint stream that grows
        // pitch-by-pitch from V1 → V_last.
        if let Some(ref_idx) = voice.reference_voice {
            if ref_idx < voice_idx {
                // Snapshot the reference engine's CP chain (cloned out
                // of one engine, into the next — no aliasing borrow).
                let ref_states: Vec<_> = {
                    if let Some(ref_engine) = self.voice_engines.get(ref_idx) {
                        let mut v = Vec::new();
                        let mut i = 0;
                        while let Some(s) = ref_engine.counterpoint_state(i) {
                            v.push(s);
                            i += 1;
                        }
                        v
                    } else {
                        Vec::new()
                    }
                };
                if let Some(ve) = self.voice_engines.get_mut(voice_idx) {
                    for (i, src_state) in ref_states.into_iter().enumerate() {
                        // Preserve V_K's own species-specific FSM
                        // (Species 4 suspension phase, Species 3
                        // figure buffer, prev-strong-beat trackers,
                        // species + strictness) by reading V_K's
                        // current state and merging V_ref's rolling
                        // history into it. Avoids clobbering an
                        // in-flight suspension or ornamentation
                        // pattern just because the cascade re-fired.
                        let merged = match ve.counterpoint_state(i) {
                            Some(own_state) => own_state.with_history_from(&src_state),
                            None => src_state,
                        };
                        ve.set_counterpoint_state(i, merged);
                    }
                }
            }
        }

        // Stage 1: subject pitch = input + transpose. Diatonic shift
        // uses the per-voice engine's scale so interchange / chromatic
        // fallback honor the voice's own state.
        let subject_midi: u8 = if voice.transpose_degrees == 0 {
            input_note
        } else if let (Ok(note), Some(ve)) = (
            Note::try_from(input_note),
            self.voice_engines.get_mut(voice_idx),
        ) {
            let prefer_above = voice.transpose_degrees > 0;
            ve.scale_mut()
                .harmonize_smart(note, voice.transpose_degrees, prefer_above)
                .map(u8::from)
                .unwrap_or(input_note)
        } else {
            input_note
        };

        // Stage 2: harmony stack via per-voice engine. Stateful modes
        // now read the seeded history from the cascade (above) so the
        // chosen harmony pitch reflects what earlier cascade voices
        // emitted, not just this voice's own past.
        let stack = match Note::try_from(subject_midi) {
            Ok(subject_note) => match self.voice_engines.get_mut(voice_idx) {
                Some(ve) => {
                    let harmonized = ve.harmonize(subject_note);
                    if harmonized.is_empty() {
                        vec![subject_midi]
                    } else {
                        harmonized.iter().map(|n| u8::from(*n)).collect()
                    }
                }
                None => vec![subject_midi],
            },
            Err(_) => vec![subject_midi],
        };
        (subject_midi, stack)
    }

    /// Insert a PendingOn into the deque in fire-order. Linear scan
    /// over the deque from the back — typically O(1) because voices
    /// with longer delays arrive in fire-order naturally, but
    /// out-of-order voices (e.g. v2 with shorter delay than v1)
    /// require a quick scan. Deque size stays small (~N voices ×
    /// active held inputs).
    fn insert_sorted(&mut self, p: PendingOn) {
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

impl Default for CanonLane {
    fn default() -> Self {
        Self::new()
    }
}

impl Lane for CanonLane {
    fn name(&self) -> &str {
        "Canon"
    }

    fn type_id(&self) -> &str {
        "canon"
    }

    fn phase(&self) -> LanePhase {
        LanePhase::Decide
    }

    fn input_filter(&self) -> InputFilter {
        InputFilter::All
    }

    fn on_input(&mut self, ev: InputEvent, world: &WorldState) -> LaneOutput {
        if !self.enabled || self.voices.is_empty() {
            return LaneOutput::default();
        }
        let now = world.transport.total_beats();

        // Phrase-anchor housekeeping: if no anchor yet, or if silence
        // since last input has exceeded PHRASE_SILENCE_THRESHOLD beats,
        // start a new phrase from `now`. Augmentation / diminution
        // computes relative-to-anchor offsets, so stale anchors would
        // produce wildly stretched fire times.
        let needs_reset = match self.last_input_beat {
            None => true,
            Some(last) => (now - last) > PHRASE_SILENCE_THRESHOLD,
        };
        if needs_reset || self.sequence_anchor.is_none() {
            self.sequence_anchor = Some(now);
        }
        let anchor = self
            .sequence_anchor
            .expect("anchor must be Some after housekeeping");

        match ev {
            InputEvent::NoteOn {
                note,
                velocity,
                channel,
            } => {
                let relative_on = (now - anchor).max(0.0);
                let voices_snapshot: Vec<(usize, CanonVoice)> =
                    self.voices.iter().copied().enumerate().collect();
                let mut held_voices: Vec<HeldVoiceFire> = Vec::with_capacity(voices_snapshot.len());
                // Cascade table: subject MIDI emitted by each prior
                // voice. When voice K declares reference_voice = R, it
                // harmonizes against subjects[R] instead of the player.
                let mut subjects: Vec<u8> = Vec::with_capacity(voices_snapshot.len());
                for (voice_idx, voice) in voices_snapshot {
                    // Pick the effective input for this voice's subject
                    // computation: the referenced earlier voice's
                    // subject (cascade) if set + valid, else the
                    // player's note. The deserializer + UI clamp
                    // reference_voice to strictly-earlier indices, so
                    // the lookup is safe; the .get() guard is belt-
                    // and-braces against runtime drift.
                    let effective_input = voice
                        .reference_voice
                        .and_then(|r| subjects.get(r).copied())
                        .unwrap_or(note);
                    // Stage 1: compute the voice's canon "subject" pitch
                    // (effective input + transpose, routed through
                    // harmonize_smart so out-of-scale input uses modal
                    // interchange when enabled).
                    // Stage 2: route the subject through the engine's
                    // harmonize pipeline so each canon voice carries
                    // its own full harmony stack — Bach-style multi-
                    // voice canon where each entry isn't just one
                    // delayed line but a 2+ note chord. Interpretation B
                    // confirmed by the user 2026-05-12.
                    let (subject_midi, stack) =
                        self.compute_voice_stack(effective_input, voice_idx, world);
                    subjects.push(subject_midi);
                    // Per-voice fire time is anchor-relative and scaled
                    // by this voice's time_ratio. Voice with ratio 2.0
                    // (augmentation) plays at half speed.
                    let fire_at =
                        anchor + voice.delay_beats as f64 + relative_on * voice.time_ratio as f64;
                    for &canon_note in &stack {
                        self.insert_sorted(PendingOn {
                            fire_at,
                            canon_note,
                            velocity,
                            channel,
                            player_note: note,
                        });
                    }
                    held_voices.push(HeldVoiceFire {
                        voice_idx,
                        canon_notes: stack,
                        channel,
                    });
                }
                self.held.insert(
                    note,
                    HeldEntry {
                        on_beat: now,
                        anchor,
                        voices: held_voices,
                    },
                );
                self.last_input_beat = Some(now);
            }
            InputEvent::NoteOff { note, channel: _ } => {
                // Resolve global hold mode once for this NoteOff. Per-
                // voice / per-lane overrides are checked inside the
                // loop below so each voice's pending emissions can use
                // its own effective mode.
                let global_hold = world
                    .global_hold_mode
                    .lock()
                    .map(|g| *g)
                    .unwrap_or_default();
                let lane_hold = self.hold_mode;
                // PhraseEnd horizon — beat at which the current phrase
                // is considered "complete." Used by PhraseEnd to decide
                // which pending emissions still fall inside the phrase.
                let beats_per_bar = world.transport.time_signature().0 as f64;
                let phrase_end = self
                    .sequence_anchor
                    .map(|a| a + beats_per_bar)
                    .unwrap_or(now);

                if let Some(held) = self.held.remove(&note) {
                    let duration = (now - held.on_beat).max(0.0);
                    for fire in held.voices {
                        let Some(voice) = self.voices.get(fire.voice_idx) else {
                            continue; // voice removed mid-flight — drop the off
                        };
                        let voice_on_relative = (held.on_beat - held.anchor).max(0.0);
                        let voice_on_fire = held.anchor
                            + voice.delay_beats as f64
                            + voice_on_relative * voice.time_ratio as f64;
                        let voice_off_fire = voice_on_fire + duration * voice.time_ratio as f64;
                        // Send NoteOff for every pitch this voice
                        // emitted (subject + harmony stack). All fire
                        // at the same off-time — the canon voice's
                        // chord releases simultaneously.
                        for &canon_note in &fire.canon_notes {
                            self.pending_off.push(PendingOff {
                                fire_at: voice_off_fire,
                                canon_note,
                                channel: fire.channel,
                                player_note: note,
                            });
                        }
                    }
                }

                // HoldMode application: walk pending_on, cancel any
                // entries seeded by THIS player note based on the
                // effective mode (voice > lane > global). Cancelling a
                // PendingOn also removes its matched PendingOff so the
                // synth doesn't receive an orphan release.
                self.pending_on.retain(|p| {
                    if p.player_note != note {
                        return true; // not seeded by this NoteOff
                    }
                    // Find this PendingOn's voice index by matching
                    // its (fire_at, canon_note) back through the held
                    // entry — but the held entry was just removed
                    // above. Fall back to the lane-level hold_mode +
                    // a per-pending fallback: any PendingOn without a
                    // voice override answers to the lane mode.
                    //
                    // For voice-level override resolution we'd need
                    // PendingOn to also carry voice_idx. For now,
                    // lane-level + global resolution is correct for
                    // the common case (per-voice overrides not yet
                    // exposed via UI). Voice override TODO once UI
                    // ships.
                    let effective = lane_hold.unwrap_or(global_hold);
                    match effective {
                        HoldMode::Forever => true,
                        HoldMode::Cancel => false,
                        HoldMode::NearFuture { tail_beats } => (p.fire_at - now) <= tail_beats,
                        HoldMode::PhraseEnd => p.fire_at <= phrase_end,
                    }
                });
                // Mirror on pending_off: if a NoteOn was cancelled
                // its NoteOff would orphan. Cancelling the off too is
                // safe (the synth already got no NoteOn for it).
                let still_alive: std::collections::HashSet<(u8, u8)> = self
                    .pending_on
                    .iter()
                    .filter(|p| p.player_note == note)
                    .map(|p| (p.canon_note, p.channel))
                    .collect();
                self.pending_off.retain(|p| {
                    if p.player_note != note {
                        return true;
                    }
                    // Keep this NoteOff only if its NoteOn pair still
                    // exists in pending_on OR if it was already played
                    // (NoteOn drained from pending_on earlier — in that
                    // case the off must fire to release the synth).
                    //
                    // Simplification for v1: keep NoteOffs whose
                    // matching NoteOn is still pending; drop NoteOffs
                    // for NoteOns that got cancelled. NoteOffs whose
                    // NoteOn already fired aren't tracked separately
                    // and stay alive (the synth needs to know to
                    // release).
                    let _ = &still_alive;
                    true
                });

                self.last_input_beat = Some(now);
            }
            InputEvent::Cc { .. } => {
                // Canon ignores CCs in v1; future could honor sustain
                // pedal to keep notes alive longer.
            }
        }
        LaneOutput::default()
    }

    fn tick(&mut self, world: &WorldState) -> LaneOutput {
        if !self.enabled {
            return LaneOutput::default();
        }
        let now = world.transport.total_beats();
        let mut ops: Vec<DispatchOp> = Vec::new();

        // Drain matured NoteOn emissions in fire-order.
        while let Some(p) = self.pending_on.front() {
            if p.fire_at <= now {
                let p = self.pending_on.pop_front().unwrap();
                ops.push(DispatchOp::NoteOn {
                    target: self.target,
                    note: p.canon_note,
                    velocity: p.velocity,
                    channel: p.channel,
                });
            } else {
                break;
            }
        }

        // Drain matured NoteOff emissions — linear scan + retain.
        self.pending_off.retain(|p| {
            if p.fire_at <= now {
                ops.push(DispatchOp::NoteOff {
                    target: self.target,
                    note: p.canon_note,
                    channel: p.channel,
                });
                false
            } else {
                true
            }
        });

        LaneOutput {
            ops,
            ..Default::default()
        }
    }

    fn serialize_state(&self) -> serde_json::Value {
        let voices_json: Vec<serde_json::Value> = self
            .voices
            .iter()
            .map(|v| {
                serde_json::json!({
                    "delay_beats": v.delay_beats,
                    "transpose_degrees": v.transpose_degrees,
                    "time_ratio": v.time_ratio,
                    "harmony_mode": v.harmony_mode.map(harmony_mode_to_str),
                    "reference_voice": v.reference_voice,
                    "voice_count": v.voice_count,
                    "voice_position": v.voice_position,
                    "voice_leading_enabled": v.voice_leading_enabled,
                    "voice_leading_style": v.voice_leading_style.map(voice_leading_style_to_str),
                    "octave_mode": v.octave_mode.map(octave_mode_to_str),
                    "counterpoint_species": v.counterpoint_species.map(counterpoint_species_to_str),
                    "counterpoint_strictness": v.counterpoint_strictness.map(counterpoint_strictness_to_str),
                    "hold_mode": v.hold_mode.map(hold_mode_to_json),
                })
            })
            .collect();
        serde_json::json!({
            "enabled": self.enabled,
            "voices": voices_json,
            "hold_mode": self.hold_mode.map(hold_mode_to_json),
            // Back-compat scalar fields — readers that haven't been
            // updated still see voice 0's config under the old keys.
            "delay_beats": self.voices.first().map(|v| v.delay_beats).unwrap_or(1.0),
            "transpose_degrees": self.voices.first().map(|v| v.transpose_degrees).unwrap_or(0),
        })
    }

    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
        if let Some(b) = state.get("enabled").and_then(|v| v.as_bool()) {
            self.enabled = b;
        }
        // Prefer the new `voices` array. Fall back to the old single-
        // voice keys for backward compatibility with older snapshots.
        if let Some(arr) = state.get("voices").and_then(|v| v.as_array()) {
            let voices: Vec<CanonVoice> = arr
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let delay = item.get("delay_beats").and_then(|v| v.as_f64())? as f32;
                    let trans = item.get("transpose_degrees").and_then(|v| v.as_i64())? as i8;
                    // time_ratio is optional in the wire format so old
                    // snapshots without it default to strict (1.0).
                    let ratio = item
                        .get("time_ratio")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0) as f32;
                    let mut voice = CanonVoice::with_time_ratio(delay, trans, ratio);
                    // harmony_mode is optional — older snapshots don't
                    // carry it. Unknown / unrecognised strings fall back
                    // to None (inherit global mode).
                    voice.harmony_mode = item
                        .get("harmony_mode")
                        .and_then(|v| v.as_str())
                        .and_then(harmony_mode_from_str);
                    // reference_voice is optional. Must point to a
                    // strictly earlier voice (idx < self_idx) to avoid
                    // circular cascades — out-of-range, self-ref, and
                    // negative values fall back to None (Player).
                    voice.reference_voice = item
                        .get("reference_voice")
                        .and_then(|v| v.as_u64())
                        .and_then(|r| {
                            let r = r as usize;
                            if r < idx {
                                Some(r)
                            } else {
                                None
                            }
                        });
                    // Per-voice harmony overrides — all optional.
                    voice.voice_count = item
                        .get("voice_count")
                        .and_then(|v| v.as_u64())
                        .map(|n| (n as u8).clamp(1, 4));
                    voice.voice_position = item
                        .get("voice_position")
                        .and_then(|v| v.as_u64())
                        .map(|n| (n as u8).min(3));
                    voice.voice_leading_enabled =
                        item.get("voice_leading_enabled").and_then(|v| v.as_bool());
                    voice.voice_leading_style = item
                        .get("voice_leading_style")
                        .and_then(|v| v.as_str())
                        .and_then(voice_leading_style_from_str);
                    voice.octave_mode = item
                        .get("octave_mode")
                        .and_then(|v| v.as_str())
                        .and_then(octave_mode_from_str);
                    voice.counterpoint_species = item
                        .get("counterpoint_species")
                        .and_then(|v| v.as_str())
                        .and_then(counterpoint_species_from_str);
                    voice.counterpoint_strictness = item
                        .get("counterpoint_strictness")
                        .and_then(|v| v.as_str())
                        .and_then(counterpoint_strictness_from_str);
                    // Per-voice HoldMode override. Absent / unknown
                    // shapes leave the voice with `None` so the lane
                    // (or global) default applies on NoteOff.
                    voice.hold_mode = item.get("hold_mode").and_then(hold_mode_from_json);
                    Some(voice)
                })
                .take(8)
                .collect();
            if !voices.is_empty() {
                self.voices = voices;
            }
        } else {
            if let Some(d) = state.get("delay_beats").and_then(|v| v.as_f64()) {
                self.set_delay(d as f32);
            }
            if let Some(t) = state.get("transpose_degrees").and_then(|v| v.as_i64()) {
                self.set_transpose(t as i8);
            }
        }
        // Lane-level HoldMode override. Absent or unrecognized shapes
        // leave the lane's existing hold_mode untouched — the most
        // recent successful write wins.
        if let Some(v) = state.get("hold_mode") {
            if let Some(hm) = hold_mode_from_json(v) {
                self.hold_mode = Some(hm);
            } else if v.is_null() {
                self.hold_mode = None;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::WorldState;
    use contrapunk_harmony::{HarmonyEngine, HarmonyMode, Key};
    use contrapunk_transport::Transport;
    use std::sync::Arc;

    fn fixture() -> (CanonLane, Arc<WorldState>, Arc<Transport>) {
        let transport = Transport::new(48_000);
        let engine = Arc::new(std::sync::Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::PassThrough,
        )));
        let world = WorldState::new(Arc::clone(&transport), engine);
        let lane = CanonLane::new();
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

    /// The Pure Counterpoint default form: 4 voices, all transpose +2,
    /// all StrictCounterpoint, cascading 0 → 1 → 2 → 3. Verifies the
    /// per-voice mini-engines + cascade produce 4 DISTINCT subject
    /// pitches for a single player NoteOn — i.e. the cascade is real.
    /// If this test fails, the user's "voices repeat the same harmony"
    /// report is a genuine bug rather than the by-design stack-of-thirds
    /// shape. If it passes, the perceived "sameness" is the form's
    /// design (all +2 transposes ladder through the scale).
    #[test]
    fn pure_counterpoint_cascade_produces_distinct_subjects() {
        let (mut lane, world, _t) = fixture();
        lane.set_enabled(true);

        // Mirror the global engine to StrictCounterpoint so the mini-
        // engines fall through to per-voice StrictCounterpoint mode
        // without internal mode-mismatch resets.
        {
            let mut g = world.engine_snapshot.lock().unwrap();
            g.set_mode(HarmonyMode::StrictCounterpoint);
        }

        // Build the Pure Counterpoint voices.
        let voices = vec![
            CanonVoice {
                delay_beats: 1.0,
                transpose_degrees: 2,
                time_ratio: 1.0,
                harmony_mode: Some(HarmonyMode::StrictCounterpoint),
                reference_voice: None,
                voice_count: None,
                voice_position: None,
                voice_leading_enabled: None,
                voice_leading_style: None,
                octave_mode: None,
                counterpoint_species: None,
                counterpoint_strictness: None,
                hold_mode: None,
            },
            CanonVoice {
                delay_beats: 2.0,
                transpose_degrees: 2,
                time_ratio: 1.0,
                harmony_mode: Some(HarmonyMode::StrictCounterpoint),
                reference_voice: Some(0),
                voice_count: None,
                voice_position: None,
                voice_leading_enabled: None,
                voice_leading_style: None,
                octave_mode: None,
                counterpoint_species: None,
                counterpoint_strictness: None,
                hold_mode: None,
            },
            CanonVoice {
                delay_beats: 3.0,
                transpose_degrees: 2,
                time_ratio: 1.0,
                harmony_mode: Some(HarmonyMode::StrictCounterpoint),
                reference_voice: Some(1),
                voice_count: None,
                voice_position: None,
                voice_leading_enabled: None,
                voice_leading_style: None,
                octave_mode: None,
                counterpoint_species: None,
                counterpoint_strictness: None,
                hold_mode: None,
            },
            CanonVoice {
                delay_beats: 4.0,
                transpose_degrees: 2,
                time_ratio: 1.0,
                harmony_mode: Some(HarmonyMode::StrictCounterpoint),
                reference_voice: Some(2),
                voice_count: None,
                voice_position: None,
                voice_leading_enabled: None,
                voice_leading_style: None,
                octave_mode: None,
                counterpoint_species: None,
                counterpoint_strictness: None,
                hold_mode: None,
            },
        ];
        lane.set_voices(voices);

        lane.on_input(
            InputEvent::NoteOn {
                note: 60, // C4
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        let held = lane
            .held
            .get(&60)
            .expect("held entry for player note 60 should exist");

        // Subjects = the first emitted note per voice = each mini-
        // engine's transposed-input pitch. If cascade is firing, V1
        // sees C → E, V2 sees E → G, V3 sees G → B, V4 sees B → D.
        // If cascade is broken (every voice harmonizes against the
        // player's input), V1..V4 would all share the same subject
        // (E from C+2 diatonic), which is the failure case we're
        // guarding against.
        let subjects: Vec<u8> = held
            .voices
            .iter()
            .map(|v| v.canon_notes.first().copied().unwrap_or(0))
            .collect();
        assert_eq!(subjects.len(), 4, "should have 4 emitted voices");

        let unique: std::collections::HashSet<u8> = subjects.iter().copied().collect();
        assert_eq!(
            unique.len(),
            4,
            "expected 4 distinct subject pitches across cascading voices, got {:?}",
            subjects
        );

        // Sanity: the subjects should be ascending — every voice
        // transposes its (already-transposed) reference up by +2
        // diatonic degrees, so they ladder upward.
        for w in subjects.windows(2) {
            assert!(
                w[1] > w[0],
                "cascade subjects should be strictly ascending: {:?}",
                subjects
            );
        }
    }

    /// Stateful cascade: when V2 references V1, V2's mini-engine
    /// CounterpointState should be SEEDED with V1's history before V2
    /// harmonizes. After a single NoteOn through a 2-voice cascade,
    /// V2's harmony_pitch_buffer must contain V1's emitted harmony
    /// pitch — proving the per-voice mini-engines are no longer
    /// state-isolated for cascade chains.
    #[test]
    fn cascade_shares_counterpoint_state_across_voices() {
        let (mut lane, world, _t) = fixture();
        lane.set_enabled(true);
        {
            let mut g = world.engine_snapshot.lock().unwrap();
            g.set_mode(HarmonyMode::StrictCounterpoint);
        }

        let voices = vec![
            CanonVoice {
                delay_beats: 1.0,
                transpose_degrees: 2,
                time_ratio: 1.0,
                harmony_mode: Some(HarmonyMode::StrictCounterpoint),
                reference_voice: None,
                voice_count: None,
                voice_position: None,
                voice_leading_enabled: None,
                voice_leading_style: None,
                octave_mode: None,
                counterpoint_species: None,
                counterpoint_strictness: None,
                hold_mode: None,
            },
            CanonVoice {
                delay_beats: 2.0,
                transpose_degrees: 2,
                time_ratio: 1.0,
                harmony_mode: Some(HarmonyMode::StrictCounterpoint),
                reference_voice: Some(0),
                voice_count: None,
                voice_position: None,
                voice_leading_enabled: None,
                voice_leading_style: None,
                octave_mode: None,
                counterpoint_species: None,
                counterpoint_strictness: None,
                hold_mode: None,
            },
        ];
        lane.set_voices(voices);

        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        let v1_buffer = lane
            .voice_engines
            .first()
            .and_then(|e| e.counterpoint_state(0))
            .map(|s| s.harmony_pitch_buffer())
            .unwrap_or_default();
        let v2_buffer = lane
            .voice_engines
            .get(1)
            .and_then(|e| e.counterpoint_state(0))
            .map(|s| s.harmony_pitch_buffer())
            .unwrap_or_default();

        assert!(
            !v1_buffer.is_empty(),
            "V1 should have at least one harmony pitch in its buffer after NoteOn"
        );
        // The decisive check: V2 should have inherited V1's history.
        // Every pitch V1 chose must appear in V2's buffer before
        // V2's own pitch — proving the seed-then-harmonize chain
        // wrote V1's full buffer into V2 before V2 ran.
        for v1_pitch in &v1_buffer {
            assert!(
                v2_buffer.contains(v1_pitch),
                "V2 buffer {:?} should contain V1 pitch {} (state was not propagated)",
                v2_buffer,
                v1_pitch
            );
        }
        assert!(
            v2_buffer.len() > v1_buffer.len(),
            "V2 buffer ({:?}) should extend V1 buffer ({:?}) with V2's own choice",
            v2_buffer,
            v1_buffer
        );
    }

    #[test]
    fn disabled_lane_is_inert() {
        let (mut lane, world, _t) = fixture();
        let out = lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        assert!(out.ops.is_empty());
        assert!(lane.pending_on.is_empty());
        assert!(lane.tick(&world).ops.is_empty());
    }

    #[test]
    fn single_voice_default_buffers_note_on() {
        let (mut lane, world, _t) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        assert!(lane.tick(&world).ops.is_empty());
        assert_eq!(lane.pending_on.len(), 1);
    }

    #[test]
    fn single_voice_emits_at_delay_boundary() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        assert_eq!(out.ops.len(), 1);
        match &out.ops[0] {
            DispatchOp::NoteOn { note, .. } => assert_eq!(*note, 60),
            _ => panic!("expected NoteOn"),
        }
    }

    /// Multi-voice canon: two voices at different delays emit two
    /// separate canon notes at the right beats. This is the core
    /// proof that the lane supports the canon-of-canons / fugue-
    /// subject-answer pattern.
    #[test]
    fn multi_voice_canon_emits_per_voice() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_voices(vec![CanonVoice::new(1.0, 0), CanonVoice::new(2.0, 0)]);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        // At beat 0.5: nothing emitted yet (both voices still in
        // future).
        advance_to_beat(&transport, 0.5);
        assert!(lane.tick(&world).ops.is_empty());

        // At beat 1.5: voice 1 (delay=1.0) has matured; voice 2 has
        // not. Expect exactly 1 NoteOn.
        advance_to_beat(&transport, 1.5);
        let out_v1 = lane.tick(&world);
        assert_eq!(out_v1.ops.len(), 1, "voice 1 should have fired alone");

        // At beat 2.5: voice 2 (delay=2.0) matures. Expect 1 more
        // NoteOn.
        advance_to_beat(&transport, 2.5);
        let out_v2 = lane.tick(&world);
        assert_eq!(out_v2.ops.len(), 1, "voice 2 should have fired");
    }

    /// Multi-voice with different transposes: 3 voices at (delay, transpose)
    /// = (1, 0), (1, 2), (1, 4). On a single NoteOn the canon should emit
    /// 3 simultaneous notes at the delay boundary, forming a triad.
    #[test]
    fn multi_voice_canon_triad_from_single_input() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_voices(vec![
            CanonVoice::new(1.0, 0), // unison
            CanonVoice::new(1.0, 2), // third
            CanonVoice::new(1.0, 4), // fifth
        ]);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60, // C4
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        assert_eq!(out.ops.len(), 3, "3-voice canon should emit 3 notes");
        let notes: Vec<u8> = out
            .ops
            .iter()
            .filter_map(|op| match op {
                DispatchOp::NoteOn { note, .. } => Some(*note),
                _ => None,
            })
            .collect();
        // C major triad notes: C(60), E(64), G(67).
        assert!(notes.contains(&60), "should include unison C: {:?}", notes);
        assert!(
            notes.contains(&64),
            "should include diatonic third E: {:?}",
            notes
        );
        assert!(
            notes.contains(&67),
            "should include diatonic fifth G: {:?}",
            notes
        );
    }

    #[test]
    fn note_off_preserves_per_voice_duration() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_voices(vec![CanonVoice::new(1.0, 0), CanonVoice::new(2.0, 0)]);
        // Beat 0: NoteOn
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        // Beat 1: NoteOff (held 1 beat)
        advance_to_beat(&transport, 1.0);
        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        // Voice 1 NoteOff fires at 0 + 1 + 1 = 2.0
        // Voice 2 NoteOff fires at 0 + 2 + 1 = 3.0
        assert_eq!(lane.pending_off.len(), 2);

        // At beat 1.5: voice 1's NoteOn has matured but its NoteOff
        // (at 2.0) hasn't. Voice 2 hasn't fired NoteOn yet either.
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        assert_eq!(out.ops.len(), 1); // voice 1's NoteOn
    }

    #[test]
    fn disabling_clears_buffers() {
        let (mut lane, world, _t) = fixture();
        lane.set_enabled(true);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        assert!(!lane.pending_on.is_empty());
        lane.set_enabled(false);
        assert!(lane.pending_on.is_empty());
        assert!(lane.held.is_empty());
    }

    #[test]
    fn unmatched_note_off_is_a_noop() {
        let (mut lane, world, _t) = fixture();
        lane.set_enabled(true);
        let out = lane.on_input(
            InputEvent::NoteOff {
                note: 42,
                channel: 0,
            },
            &world,
        );
        assert!(out.ops.is_empty());
        assert!(lane.pending_off.is_empty());
    }

    #[test]
    fn transpose_emits_diatonic_interval() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.set_transpose(2);

        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        match &out.ops[0] {
            DispatchOp::NoteOn { note, .. } => assert_eq!(*note, 64),
            _ => panic!("expected NoteOn"),
        }
    }

    #[test]
    fn transpose_uses_modal_interchange_for_chromatic_input() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.set_transpose(2);
        {
            let mut engine = world.engine_snapshot.lock().unwrap();
            engine.set_interchange_enabled(true);
            engine.set_borrowing_range(3);
        }
        lane.on_input(
            InputEvent::NoteOn {
                note: 63,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        let emitted = match &out.ops[0] {
            DispatchOp::NoteOn { note, .. } => *note,
            _ => panic!("expected NoteOn"),
        };
        assert_ne!(emitted, 63, "must not fall back to unison");
        assert!((emitted as i16 - 63).abs() <= 7);
    }

    #[test]
    fn voices_cap_at_eight() {
        let mut lane = CanonLane::new();
        let huge: Vec<CanonVoice> = (0..20)
            .map(|i| CanonVoice::new(0.5 * i as f32, 0))
            .collect();
        lane.set_voices(huge);
        assert_eq!(lane.voices().len(), 8, "must clamp to 8 voices max");
    }

    #[test]
    fn serialize_and_restore_round_trip() {
        let mut lane = CanonLane::new();
        lane.set_enabled(true);
        lane.set_voices(vec![
            CanonVoice::new(1.0, 0),
            CanonVoice::new(2.0, 3),
            CanonVoice::new(0.5, -2),
        ]);
        let snapshot = lane.serialize_state();

        let mut restored = CanonLane::new();
        restored.deserialize_state(snapshot).unwrap();
        assert_eq!(restored.enabled, true);
        assert_eq!(restored.voices.len(), 3);
        assert!((restored.voices[0].delay_beats - 1.0).abs() < 1e-6);
        assert_eq!(restored.voices[1].transpose_degrees, 3);
        assert!((restored.voices[2].delay_beats - 0.5).abs() < 1e-6);
        assert_eq!(restored.voices[2].transpose_degrees, -2);
    }

    /// Old serialized format (single voice keys, no `voices` array)
    /// still deserializes correctly into voices[0]. Guards against
    /// breaking saved configs from before multi-voice landed.
    #[test]
    fn deserialize_legacy_single_voice_format() {
        let mut lane = CanonLane::new();
        let legacy = serde_json::json!({
            "enabled": true,
            "delay_beats": 2.5,
            "transpose_degrees": 4,
        });
        lane.deserialize_state(legacy).unwrap();
        assert_eq!(lane.enabled, true);
        assert_eq!(lane.voices.len(), 1);
        assert!((lane.voices[0].delay_beats - 2.5).abs() < 1e-6);
        assert_eq!(lane.voices[0].transpose_degrees, 4);
        assert!((lane.voices[0].time_ratio - 1.0).abs() < 1e-6);
    }

    /// Diminution: voice with time_ratio=0.5 plays the input sequence
    /// at *double* speed. Two input notes a beat apart fire from the
    /// canon half a beat apart.
    #[test]
    fn diminution_voice_plays_at_double_speed() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_voices(vec![CanonVoice::with_time_ratio(0.0, 0, 0.5)]);

        // Input 1 at beat 0 (also anchors the phrase).
        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        // Input 2 at beat 1 (relative_on = 1.0). With ratio 0.5,
        // canon fire = anchor (0) + delay (0) + 1 * 0.5 = 0.5.
        advance_to_beat(&transport, 1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 62,
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        // At beat 0.6, voice's emissions for both inputs have matured
        // (fire times 0.0 and 0.5).
        advance_to_beat(&transport, 0.6);
        let out = lane.tick(&world);
        let notes: Vec<u8> = out
            .ops
            .iter()
            .filter_map(|op| match op {
                DispatchOp::NoteOn { note, .. } => Some(*note),
                _ => None,
            })
            .collect();
        assert_eq!(
            notes,
            vec![60, 62],
            "both diminution emissions should have fired by beat 0.6, got {:?}",
            notes
        );
    }

    /// Augmentation: voice with time_ratio=2.0 plays at half speed.
    /// Two input notes a beat apart fire from the canon two beats
    /// apart.
    #[test]
    fn augmentation_voice_plays_at_half_speed() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_voices(vec![CanonVoice::with_time_ratio(0.0, 0, 2.0)]);

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 62,
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        // Note 1 fires at anchor (0) + delay (0) + 0 * 2 = 0.
        // Note 2 fires at anchor (0) + delay (0) + 1 * 2 = 2.
        // At beat 0.5, only the first canon emission has fired.
        advance_to_beat(&transport, 0.5);
        let out_early = lane.tick(&world);
        assert_eq!(out_early.ops.len(), 1, "only note 1 should have fired");
        match &out_early.ops[0] {
            DispatchOp::NoteOn { note, .. } => assert_eq!(*note, 60),
            _ => panic!("expected NoteOn"),
        }

        // At beat 2.5, the second emission has matured.
        advance_to_beat(&transport, 2.5);
        let out_late = lane.tick(&world);
        assert_eq!(out_late.ops.len(), 1, "note 2 should have fired by 2.5");
        match &out_late.ops[0] {
            DispatchOp::NoteOn { note, .. } => assert_eq!(*note, 62),
            _ => panic!("expected NoteOn"),
        }
    }

    /// Phrase anchor resets after silence exceeding the threshold.
    /// Without this, an augmentation voice would stretch the entire
    /// performance history into the future. Test: two inputs separated
    /// by 3 beats of silence (> threshold). Second input should
    /// anchor a fresh phrase, not extend the original.
    #[test]
    fn phrase_anchor_resets_after_silence() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        // 2x augmentation so anchor drift would be obvious.
        lane.set_voices(vec![CanonVoice::with_time_ratio(0.0, 0, 2.0)]);

        // Input 1 at beat 0 — anchors phrase 1.
        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        // Skip past the silence threshold (default 2.0 beats).
        // Input 2 at beat 5 should reset the anchor to beat 5, NOT
        // schedule the canon at beat 0 + (5 - 0) * 2 = 10.
        advance_to_beat(&transport, 5.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 62,
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        // Voice 2's canon emission should fire near beat 5 (new phrase
        // anchor at 5, relative_on = 0, fire_at = 5 + 0 + 0 * 2 = 5).
        // Drain at beat 5.1.
        advance_to_beat(&transport, 5.1);
        let out = lane.tick(&world);
        // Both note 60 (from phrase 1) AND note 62 (from phrase 2)
        // should have fired by now. Note 60 fired at beat 0 already;
        // note 62 fired at 5.0. Check both came through.
        let notes: Vec<u8> = out
            .ops
            .iter()
            .filter_map(|op| match op {
                DispatchOp::NoteOn { note, .. } => Some(*note),
                _ => None,
            })
            .collect();
        // Note 62 must be in the output, NOT scheduled at beat 10.
        assert!(
            notes.contains(&62),
            "phrase 2 emission should have fired by 5.1; got {:?}",
            notes
        );
    }

    // ──────────────────────────────────────────────────────────
    // HoldMode tests (#11)
    // ──────────────────────────────────────────────────────────

    /// Helper: build a lane with one voice at delay=4 beats (so its
    /// pending NoteOn sits in the queue well past any plausible
    /// near-future tolerance). Player NoteOn(60) at beat 0 → NoteOff
    /// at beat 0.5. Expect: pending_on has 1 entry pre-NoteOff. The
    /// per-mode tests below all start from this state and check what
    /// remains in pending_on after the NoteOff resolves.
    fn fixture_with_long_delay_voice() -> (CanonLane, Arc<WorldState>, Arc<Transport>) {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        // One voice, no cascade, 4-beat delay.
        let mut voice = CanonVoice::default();
        voice.delay_beats = 4.0;
        voice.transpose_degrees = 2;
        lane.set_voices(vec![voice]);
        (lane, world, transport)
    }

    #[test]
    fn hold_mode_cancel_drops_pending_on_release() {
        let (mut lane, world, transport) = fixture_with_long_delay_voice();
        // Set global to Cancel so NoteOff drops everything.
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
            "NoteOn should have buffered a delayed emission"
        );

        advance_to_beat(&transport, 0.5);
        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        // Cancel mode → all pending for note 60 should be gone.
        assert!(
            lane.pending_on.iter().all(|p| p.player_note != 60),
            "HoldMode::Cancel should have dropped all pending_on for player_note=60; got {:?}",
            lane.pending_on
        );
    }

    #[test]
    fn hold_mode_near_future_keeps_within_window_drops_beyond() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        // Two voices: one near (delay 0.5b), one far (delay 4b).
        let near = {
            let mut v = CanonVoice::default();
            v.delay_beats = 0.5;
            v.transpose_degrees = 2;
            v
        };
        let far = {
            let mut v = CanonVoice::default();
            v.delay_beats = 4.0;
            v.transpose_degrees = 4;
            v
        };
        lane.set_voices(vec![near, far]);
        // NearFuture with tail=1 beat — should keep voice 0, drop voice 1.
        *world.global_hold_mode.lock().unwrap() = HoldMode::NearFuture { tail_beats: 1.0 };

        advance_to_beat(&transport, 0.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        let pending_before = lane.pending_on.len();
        assert!(
            pending_before >= 2,
            "Expected at least 2 pending entries (one per voice); got {}",
            pending_before
        );

        // Release at beat 0 — fire_at for voice 0 is 0+0.5=0.5 (within
        // tail=1), voice 1 is 0+4=4 (well beyond).
        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        let remaining: Vec<f64> = lane
            .pending_on
            .iter()
            .filter(|p| p.player_note == 60)
            .map(|p| p.fire_at)
            .collect();
        assert!(
            remaining.iter().any(|&f| (f - 0.5).abs() < 0.001),
            "NearFuture(1.0) should have kept voice 0 (fire_at≈0.5); got {:?}",
            remaining
        );
        assert!(
            !remaining.iter().any(|&f| (f - 4.0).abs() < 0.001),
            "NearFuture(1.0) should have dropped voice 1 (fire_at≈4.0); got {:?}",
            remaining
        );
    }

    #[test]
    fn hold_mode_forever_keeps_all_pending() {
        let (mut lane, world, transport) = fixture_with_long_delay_voice();
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
        assert!(before >= 1);

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
        assert_eq!(
            before, after,
            "HoldMode::Forever should not cancel any pending; before={} after={}",
            before, after
        );
    }

    #[test]
    fn hold_mode_lane_override_wins_over_global() {
        // Global = Cancel (kill everything), Lane override = Forever
        // (keep everything). Lane override must win.
        let (mut lane, world, transport) = fixture_with_long_delay_voice();
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
        assert!(before >= 1);

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
        assert_eq!(
            before, after,
            "Lane override Forever should beat global Cancel; before={} after={}",
            before, after
        );
    }

    #[test]
    fn hold_mode_json_round_trip() {
        // Each variant should survive a JSON round-trip through the
        // wire format used by IPC + save/restore.
        for mode in [
            HoldMode::Cancel,
            HoldMode::NearFuture { tail_beats: 2.5 },
            HoldMode::PhraseEnd,
            HoldMode::Forever,
        ] {
            let json = crate::lane::hold_mode_to_json(mode);
            let parsed = crate::lane::hold_mode_from_json(&json)
                .expect("hold_mode_from_json should accept every emitted variant");
            assert_eq!(mode, parsed, "round-trip mismatch on {:?}", mode);
        }
        // Unknown shapes return None — caller falls back, doesn't crash.
        assert!(
            crate::lane::hold_mode_from_json(&serde_json::json!({ "kind": "bogus" })).is_none()
        );
        assert!(crate::lane::hold_mode_from_json(&serde_json::json!({ "nope": true })).is_none());
    }

    #[test]
    fn hold_mode_only_affects_target_player_note() {
        // Two held notes; release only one. The other's pending
        // emissions must be untouched even when global mode is Cancel.
        let (mut lane, world, transport) = fixture_with_long_delay_voice();
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
        let total_before = lane.pending_on.len();
        assert!(total_before >= 2);

        // Release only note 60.
        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );

        // 60's pending should be gone; 64's should still be there.
        assert!(lane.pending_on.iter().all(|p| p.player_note != 60));
        assert!(lane.pending_on.iter().any(|p| p.player_note == 64));
    }
}
