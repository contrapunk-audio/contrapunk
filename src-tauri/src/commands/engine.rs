//! Tauri commands for engine routing control and real-time note state.
//!
//! Handles starting/stopping the MIDI router thread and emitting
//! real-time note-update events to the frontend.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use wmidi::{Channel, MidiMessage, Note, Velocity};

use contrapunk::audio::guitar_input::GuitarInputConfig;
use contrapunk::chord::chord_display_with_analysis;
use contrapunk::elixir::{SynthEvent, SynthEventSender};
use contrapunk::harmony::HarmonyEngine;
use contrapunk::midi::input::connect_input;
use contrapunk::midi::output::OutputRouter;
use contrapunk::slide::{
    SlideConfig, SlideRole, SlideRuntime, SlideSettings, SlideSlot, SlideTelemetry, SlideTravel,
    MAX_SLIDE_VOICES,
};
use contrapunk::transport::Transport;
use contrapunk_companion::{BeatTickScheduler, InputOrigin, LoopMidiEvent, OriginMidiEvent};

use crate::guitar_bridge::GuitarBridge;
use crate::state::{
    AppState, PendingRouteChanges, VoiceOutputRoutes, VoiceOutputTarget, VoiceRouteId,
};

/// Virtual input sentinels — must match the values in MidiDevices.svelte.
const VIRTUAL_COMPUTER_KEYBOARD: usize = 999_998;
const GUITAR_AUDIO_SENTINEL: usize = 999_997;
const VIRTUAL_TONE_SOURCE: usize = 999_996;

const MIX_INPUT: u8 = 0;
const MIX_HARMONY: u8 = 1;
const MIX_CANON: u8 = 2;
const MIX_COUNTERPOINT: u8 = 3;

fn main_voice_route(result_index: usize, arrangement_slot: u8) -> VoiceRouteId {
    if result_index == 0 {
        VoiceRouteId::Input
    } else {
        VoiceRouteId::Harmony {
            slot: arrangement_slot,
        }
    }
}

fn companion_voice_route(lane: &str, voice_slot: u8) -> VoiceRouteId {
    match lane {
        "canon" => VoiceRouteId::Canon { voice: voice_slot },
        "counterpoint" => VoiceRouteId::Counterpoint { voice: voice_slot },
        "pattern_low" => VoiceRouteId::PatternLow,
        "pattern_counter" => VoiceRouteId::PatternCounter,
        _ => VoiceRouteId::Harmony { slot: voice_slot },
    }
}

type NoteCounts = HashMap<u8, u32>;
type RoutedNoteKey = (VoiceOutputTarget, u8, u8, u8, u8);

#[derive(Default)]
struct RoutedNoteCounts {
    physical: HashMap<RoutedNoteKey, u32>,
    routes: HashMap<(VoiceRouteId, RoutedNoteKey), u32>,
}

impl RoutedNoteCounts {
    fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.physical.len()
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.physical.is_empty() && self.routes.is_empty()
    }

    fn clear(&mut self) {
        self.physical.clear();
        self.routes.clear();
    }

    fn note_on(&mut self, route: VoiceRouteId, key: RoutedNoteKey) -> bool {
        *self.routes.entry((route, key)).or_insert(0) += 1;
        let first = !self.physical.contains_key(&key);
        *self.physical.entry(key).or_insert(0) += 1;
        first
    }

    fn owns(&self, route: VoiceRouteId, key: RoutedNoteKey) -> bool {
        self.routes.contains_key(&(route, key))
    }

    fn note_off(&mut self, route: VoiceRouteId, key: RoutedNoteKey) -> bool {
        let Some(route_count) = self.routes.get_mut(&(route, key)) else {
            return false;
        };
        *route_count = route_count.saturating_sub(1);
        if *route_count == 0 {
            self.routes.remove(&(route, key));
        }
        self.decrement_physical(key, 1)
    }

    fn remove_route(&mut self, route: VoiceRouteId) -> Vec<RoutedNoteKey> {
        let mut owned: Vec<_> = self
            .routes
            .iter()
            .filter_map(|(&(owner_route, key), &count)| {
                (owner_route == route).then_some((key, count))
            })
            .collect();
        owned.sort_unstable_by_key(|(key, _)| *key);
        let mut released = Vec::new();
        for (key, count) in owned {
            self.routes.remove(&(route, key));
            if self.decrement_physical(key, count) {
                released.push(key);
            }
        }
        released
    }

    fn remove_harmony_note(&mut self, note: u8) -> Vec<RoutedNoteKey> {
        let mut owners: Vec<_> = self
            .routes
            .iter()
            .filter_map(|(&(route, key), &count)| {
                (key.2 == note && matches!(route, VoiceRouteId::Harmony { .. }))
                    .then_some((route, key, count))
            })
            .collect();
        owners.sort_unstable_by_key(|(route, key, _)| (*route, *key));
        let mut released = Vec::new();
        for (route, key, count) in owners {
            self.routes.remove(&(route, key));
            if self.decrement_physical(key, count) {
                released.push(key);
            }
        }
        released.sort_unstable();
        released.dedup();
        released
    }

    fn remove_owned(&mut self, owned: &mut Self) -> Vec<RoutedNoteKey> {
        let mut owners: Vec<_> = owned.routes.drain().collect();
        owners.sort_unstable_by_key(|((route, key), _)| (*route, *key));
        let mut released = Vec::new();
        for ((route, key), count) in owners {
            if let Some(total) = self.routes.get_mut(&(route, key)) {
                *total = total.saturating_sub(count);
                if *total == 0 {
                    self.routes.remove(&(route, key));
                }
            }
            if self.decrement_physical(key, count) {
                released.push(key);
            }
        }
        owned.physical.clear();
        released.sort_unstable();
        released.dedup();
        released
    }

    fn drain(&mut self) -> Vec<RoutedNoteKey> {
        let mut keys: Vec<_> = self.physical.drain().map(|(key, _)| key).collect();
        keys.sort_unstable();
        self.routes.clear();
        keys
    }

    fn decrement_physical(&mut self, key: RoutedNoteKey, count: u32) -> bool {
        let Some(total) = self.physical.get_mut(&key) else {
            return false;
        };
        *total = total.saturating_sub(count);
        if *total > 0 {
            return false;
        }
        self.physical.remove(&key);
        true
    }
}

type SustainOwner = (InputOrigin, VoiceOutputTarget, u8);
type SustainOwners = HashSet<SustainOwner>;

const MIDI_SLIDE_BEND_RANGE_SEMITONES: f32 = 48.0;
const MIDI_INPUT_BEND_RANGE_SEMITONES: f32 = 2.0;
const MIDI_SLIDE_TIMEBASE_HZ: f32 = 1_000_000.0;
const MIDI_SLIDE_UPDATE_INTERVAL: Duration = Duration::from_millis(10);
const CC_PANIC_REBROADCAST_GUARD: Duration = Duration::from_millis(250);

#[derive(Clone, Copy)]
struct MidiSlideVoice {
    id: u64,
    port: usize,
    channel: u8,
    note: u8,
    target_hz: f32,
    key_down: bool,
}

#[derive(Clone, Copy)]
struct MidiBendUpdate {
    id: u64,
    port: usize,
    channel: u8,
    note: u8,
    frequency_hz: f32,
}

struct MidiSlideRuntime {
    slide: SlideRuntime,
    voices: Vec<MidiSlideVoice>,
    configured_channels: HashSet<(usize, u8)>,
    sustained_channels: HashSet<(usize, u8)>,
    input_bends: HashMap<(usize, u8), f32>,
    last_bends: HashMap<(usize, u8), u16>,
    telemetry: Arc<SlideTelemetry>,
    next_id: u64,
    last_tick: Instant,
    detune_cents: i32,
}

impl MidiSlideRuntime {
    fn new(detune_cents: i32, telemetry: Arc<SlideTelemetry>) -> Self {
        Self {
            slide: SlideRuntime::new(),
            voices: Vec::with_capacity(MAX_SLIDE_VOICES),
            configured_channels: HashSet::new(),
            sustained_channels: HashSet::new(),
            input_bends: HashMap::new(),
            last_bends: HashMap::new(),
            telemetry,
            next_id: 1 << 63,
            last_tick: Instant::now(),
            detune_cents,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_note_on(
        &mut self,
        port: usize,
        channel: u8,
        note: u8,
        target_hz: f32,
        slot: SlideSlot,
        settings: SlideSettings,
        output: &mut OutputRouter,
    ) {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == u64::MAX {
            self.next_id = 0;
        }
        let initial_hz = self
            .slide
            .note_on(id, slot, target_hz, settings, MIDI_SLIDE_TIMEBASE_HZ);
        self.voices.push(MidiSlideVoice {
            id,
            port,
            channel,
            note,
            target_hz,
            key_down: true,
        });

        let slide_enabled = !matches!(settings.travel, SlideTravel::Off);
        if slide_enabled {
            self.configure_channel(port, channel, output);
        }
        self.send_frequency_bend(port, channel, note, initial_hz, output);
        self.telemetry.publish(&self.slide);
    }

    fn note_off(&mut self, port: usize, channel: u8, note: u8, output: &mut OutputRouter) {
        let Some(index) = self.voices.iter().position(|voice| {
            voice.port == port && voice.channel == channel && voice.note == note && voice.key_down
        }) else {
            return;
        };
        if self.sustained_channels.contains(&(port, channel)) {
            self.voices[index].key_down = false;
            return;
        }
        let voice = self.voices.remove(index);
        self.slide.note_off(voice.id);
        self.restore_channel(port, channel, output);
        self.telemetry.publish(&self.slide);
    }

    fn set_sustain(&mut self, port: usize, channel: u8, on: bool, output: &mut OutputRouter) {
        if on {
            self.sustained_channels.insert((port, channel));
            return;
        }
        if !self.sustained_channels.remove(&(port, channel)) {
            return;
        }
        let mut released = Vec::new();
        self.voices.retain(|voice| {
            if voice.port == port && voice.channel == channel && !voice.key_down {
                released.push(voice.id);
                false
            } else {
                true
            }
        });
        for id in released {
            self.slide.note_off(id);
        }
        self.restore_channel(port, channel, output);
        self.telemetry.publish(&self.slide);
    }

    fn tick(&mut self, output: &mut OutputRouter) {
        let elapsed = self.last_tick.elapsed();
        if elapsed < MIDI_SLIDE_UPDATE_INTERVAL {
            return;
        }
        self.last_tick = Instant::now();
        self.slide
            .advance(elapsed.as_micros().min(u128::from(u32::MAX)) as usize);

        let mut updates: [Option<MidiBendUpdate>; MAX_SLIDE_VOICES] = [None; MAX_SLIDE_VOICES];
        let voices = &self.voices;
        self.slide.for_each_moving(|id, frequency_hz| {
            let Some(voice) = voices.iter().find(|voice| voice.id == id) else {
                return;
            };
            if voices.iter().any(|newer| {
                newer.port == voice.port && newer.channel == voice.channel && newer.id > voice.id
            }) {
                return;
            }
            let update = MidiBendUpdate {
                id,
                port: voice.port,
                channel: voice.channel,
                note: voice.note,
                frequency_hz,
            };
            if let Some(existing) = updates
                .iter_mut()
                .flatten()
                .find(|existing| existing.port == update.port && existing.channel == update.channel)
            {
                if update.id > existing.id {
                    *existing = update;
                }
            } else if let Some(slot) = updates.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(update);
            }
        });
        for update in updates.into_iter().flatten() {
            self.send_frequency_bend(
                update.port,
                update.channel,
                update.note,
                update.frequency_hz,
                output,
            );
        }
        self.telemetry.publish(&self.slide);
        self.slide.finish_completed();
    }

    fn set_input_bend(
        &mut self,
        port: usize,
        channel: u8,
        semitones: f32,
        output: &mut OutputRouter,
    ) {
        self.input_bends.insert((port, channel), semitones);
        self.restore_channel(port, channel, output);
    }

    fn set_detune(&mut self, cents: i32, output: &mut OutputRouter) {
        self.detune_cents = cents;
        let mut channels: Vec<_> = self
            .last_bends
            .keys()
            .chain(self.configured_channels.iter())
            .copied()
            .collect();
        channels.sort_unstable();
        channels.dedup();
        for (port, channel) in channels {
            self.restore_channel(port, channel, output);
        }
    }

    fn current_frequency(&self, voice: MidiSlideVoice) -> f32 {
        let mut frequency_hz = voice.target_hz;
        self.slide.for_each_moving(|id, current| {
            if id == voice.id {
                frequency_hz = current;
            }
        });
        frequency_hz
    }

    fn restore_channel(&mut self, port: usize, channel: u8, output: &mut OutputRouter) {
        if let Some(voice) = self
            .voices
            .iter()
            .filter(|voice| voice.port == port && voice.channel == channel)
            .max_by_key(|voice| voice.id)
            .copied()
        {
            let frequency_hz = self.current_frequency(voice);
            self.send_frequency_bend(port, channel, voice.note, frequency_hz, output);
        } else {
            self.send_semitone_bend(port, channel, 0.0, output);
        }
    }

    fn clear(&mut self, output: &mut OutputRouter) {
        let mut channels: Vec<_> = self
            .last_bends
            .keys()
            .chain(self.configured_channels.iter())
            .copied()
            .collect();
        channels.sort_unstable();
        channels.dedup();
        for (port, channel) in channels {
            self.send_bend(port, channel, 8_192, output);
            if self.configured_channels.contains(&(port, channel)) {
                for message in midi_bend_range_messages(channel, 2) {
                    if let Err(error) = output.send_to_device_port(port, &message) {
                        eprintln!("[midi] could not restore bend range on device {port}: {error}");
                    }
                }
            }
        }
        self.slide.clear();
        self.voices.clear();
        self.configured_channels.clear();
        self.sustained_channels.clear();
        self.input_bends.clear();
        self.last_bends.clear();
        self.last_tick = Instant::now();
        self.telemetry.publish(&self.slide);
    }

    fn configure_channel(&mut self, port: usize, channel: u8, output: &mut OutputRouter) {
        if !self.configured_channels.insert((port, channel)) {
            return;
        }
        for message in midi_bend_range_messages(channel, MIDI_SLIDE_BEND_RANGE_SEMITONES as u8) {
            if let Err(error) = output.send_to_device_port(port, &message) {
                eprintln!("[midi] could not configure bend range on device {port}: {error}");
            }
        }
        self.last_bends.remove(&(port, channel));
    }

    fn send_frequency_bend(
        &mut self,
        port: usize,
        channel: u8,
        note: u8,
        frequency_hz: f32,
        output: &mut OutputRouter,
    ) {
        self.send_semitone_bend(
            port,
            channel,
            midi_frequency_semitones(note, frequency_hz),
            output,
        );
    }

    fn send_semitone_bend(
        &mut self,
        port: usize,
        channel: u8,
        semitones: f32,
        output: &mut OutputRouter,
    ) {
        let range = if self.configured_channels.contains(&(port, channel)) {
            MIDI_SLIDE_BEND_RANGE_SEMITONES
        } else {
            MIDI_INPUT_BEND_RANGE_SEMITONES
        };
        let composed = semitones
            + self.detune_cents as f32 / 100.0
            + self
                .input_bends
                .get(&(port, channel))
                .copied()
                .unwrap_or(0.0);
        self.send_bend(port, channel, midi_bend_value(composed, range), output);
    }

    fn send_bend(&mut self, port: usize, channel: u8, value: u16, output: &mut OutputRouter) {
        if self.last_bends.get(&(port, channel)) == Some(&value) {
            return;
        }
        self.last_bends.insert((port, channel), value);
        let message = [
            0xe0 | (channel & 0x0f),
            (value & 0x7f) as u8,
            ((value >> 7) & 0x7f) as u8,
        ];
        if let Err(error) = output.send_to_device_port(port, &message) {
            eprintln!("[midi] could not send pitch bend to device {port}: {error}");
        }
    }
}

fn should_rebroadcast_cc_panic(last: Option<Instant>, now: Instant) -> bool {
    last.map(|last| now.duration_since(last) >= CC_PANIC_REBROADCAST_GUARD)
        .unwrap_or(true)
}

fn midi_bend_range_messages(channel: u8, semitones: u8) -> [[u8; 3]; 6] {
    let status = 0xb0 | (channel & 0x0f);
    [
        [status, 101, 0],
        [status, 100, 0],
        [status, 6, semitones],
        [status, 38, 0],
        [status, 101, 127],
        [status, 100, 127],
    ]
}

fn midi_frequency_semitones(note: u8, frequency_hz: f32) -> f32 {
    if frequency_hz.is_finite() && frequency_hz > 0.0 {
        12.0 * (frequency_hz / standard_frequency(note)).log2()
    } else {
        0.0
    }
}

fn midi_input_bend_semitones(value: u16) -> f32 {
    (value as f32 - 8_192.0) / 8_192.0 * MIDI_INPUT_BEND_RANGE_SEMITONES
}

fn midi_bend_value(semitones: f32, range_semitones: f32) -> u16 {
    (8_192.0 + semitones / range_semitones * 8_192.0)
        .round()
        .clamp(0.0, 16_383.0) as u16
}

#[derive(Clone, Copy, Debug)]
struct LoopOwnedVoice {
    route: VoiceRouteId,
    key: RoutedNoteKey,
    target: VoiceOutputTarget,
    mix_group: u8,
    channel: u8,
    note: u8,
    slide_slot: SlideSlot,
}

type LoopSourceFrames = HashMap<(u8, u8), VecDeque<Vec<LoopOwnedVoice>>>;

fn count_note_on(notes: &mut NoteCounts, note: u8) {
    let count = notes.entry(note).or_insert(0);
    *count = count.saturating_add(1);
}

fn count_note_off(notes: &mut NoteCounts, note: u8) {
    if let Some(count) = notes.get_mut(&note) {
        *count = count.saturating_sub(1);
        if *count == 0 {
            notes.remove(&note);
        }
    }
}

fn routed_note_key(
    target: VoiceOutputTarget,
    channel: u8,
    note: u8,
    mix_group: u8,
    voice_slot: u8,
) -> RoutedNoteKey {
    let (channel, mix_group, voice_slot) = if target == VoiceOutputTarget::Synth {
        (0, mix_group, voice_slot)
    } else {
        (channel, MIX_INPUT, 0)
    };
    (target, channel, note, mix_group, voice_slot)
}

#[cfg(test)]
fn count_routed_note_on(
    notes: &mut RoutedNoteCounts,
    route: VoiceRouteId,
    key: RoutedNoteKey,
) -> bool {
    notes.note_on(route, key)
}

#[cfg(test)]
fn count_routed_note_off(
    notes: &mut RoutedNoteCounts,
    route: VoiceRouteId,
    key: RoutedNoteKey,
) -> bool {
    notes.note_off(route, key)
}

fn count_origin_note_on(
    origin: InputOrigin,
    all: &mut RoutedNoteCounts,
    loop_owned: &mut RoutedNoteCounts,
    route: VoiceRouteId,
    key: RoutedNoteKey,
) -> bool {
    if origin == InputOrigin::Loop {
        loop_owned.note_on(route, key);
    }
    all.note_on(route, key)
}

fn count_origin_note_off(
    origin: InputOrigin,
    all: &mut RoutedNoteCounts,
    loop_owned: &mut RoutedNoteCounts,
    route: VoiceRouteId,
    key: RoutedNoteKey,
) -> bool {
    if origin == InputOrigin::Loop {
        if !loop_owned.owns(route, key) {
            return false;
        }
        loop_owned.note_off(route, key);
    }
    all.note_off(route, key)
}

/// Payload for the "note-update" Tauri event.
#[derive(Clone, Serialize)]
pub struct NoteUpdatePayload {
    pub input_notes: Vec<u8>,
    pub harmony_notes: Vec<u8>,
    pub borrowed_notes: Vec<u8>,
    pub chord_name: String,
    pub last_borrowed_from: String,
    pub current_key: String,
    /// Notes currently sounding from the Companion's canon lane.
    /// Kept separate so source-aware visualizations do not confuse
    /// imitative entries with generic harmonic support.
    pub canon_notes: Vec<u8>,
    /// Notes currently sounding from the Companion's counterpoint lane.
    /// Same source-attribution role as `canon_notes`.
    pub counterpoint_notes: Vec<u8>,
    pub phrase: contrapunk_companion::PhraseSnapshot,
}

/// Payload for the "guitar-signal" Tauri event (UI signal feedback).
#[derive(Clone, Serialize)]
pub struct GuitarSignalPayload {
    pub rms: f32,
    pub frequency: Option<f32>,
    pub clarity: f32,
    pub note_state: u8,
    pub note_name: String,
    pub midi_note: u8,
}

/// Payload for the "knob-cc-raw" Tauri event — every Control Change
/// message that arrives on the active MIDI input is forwarded to the
/// frontend, which resolves the CC number to a Performance-view knob
/// index via the user's MIDI Learn mapping (`contrapunk-knob-cc-map`
/// in localStorage). The router thread no longer hardcodes a CC →
/// knob-index table; that lives entirely in the UI now so users can
/// rebind any controller without a backend rebuild.
#[derive(Clone, Serialize)]
pub struct KnobCcRawPayload {
    /// MIDI CC number (0-127).
    pub cc: u8,
    /// Normalized 0.0..1.0 value (CC 0-127 / 127).
    pub value: f32,
}

/// Inject a Note-On event into the active router pipeline.
///
/// Used by virtual inputs (e.g. Computer Keyboard) in the UI. Only has
/// effect while routing is active; returns an error otherwise.
#[tauri::command]
pub fn inject_note_on(
    note: u8,
    velocity: Option<u8>,
    state: State<AppState>,
) -> Result<Vec<u8>, String> {
    let tx_guard = state.router_tx.lock().map_err(|e| e.to_string())?;
    let tx = tx_guard.as_ref().ok_or("Routing not active")?;
    let vel = velocity.unwrap_or(100).clamp(1, 127);
    tx.send(vec![0x90, note, vel]).map_err(|e| e.to_string())?;
    Ok(vec![note])
}

/// Inject a Note-Off event into the active router pipeline.
#[tauri::command]
pub fn inject_note_off(note: u8, state: State<AppState>) -> Result<Vec<u8>, String> {
    let tx_guard = state.router_tx.lock().map_err(|e| e.to_string())?;
    let tx = tx_guard.as_ref().ok_or("Routing not active")?;
    tx.send(vec![0x80, note, 0]).map_err(|e| e.to_string())?;
    Ok(vec![note])
}

/// Request the router's tracked NoteOff/CC123 drain and silence the
/// built-in synth immediately. Safe when routing is already stopped.
pub(crate) fn request_all_notes_off(state: &AppState) {
    state
        .looper
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .transport_discontinuity();
    let _ = state.synth_tx.send(SynthEvent::AllNotesOff);
    state
        .companion
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .reset_runtime();

    // Route a real CC123 through the active router so it can release
    // tracked external MIDI, clear UI note ownership, and reset the
    // HarmonyEngine. Transport Stop/Reset use this same path; merely
    // freezing the beat clock would strand future Canon NoteOffs.
    let delivered_to_router = state
        .router_tx
        .lock()
        .ok()
        .and_then(|tx| tx.as_ref().map(|tx| tx.send(vec![0xB0, 123, 0]).is_ok()))
        .unwrap_or(false);
    if !delivered_to_router {
        state
            .engine
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear_active_notes();
    }
}

#[tauri::command]
pub fn panic_all_notes_off(state: State<AppState>) {
    request_all_notes_off(&state);
}

/// Starts MIDI routing from the specified input to the specified outputs.
///
/// Spawns a background router thread that processes MIDI messages through
/// the harmony engine and emits "note-update" events at ~30fps.
#[tauri::command]
pub fn start_routing(
    input_idx: usize,
    output_indices: Vec<usize>,
    app_handle: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    // Check if already running
    if state.is_running.load(Ordering::SeqCst) {
        return Err("Routing is already active".to_string());
    }

    // Empty output_indices is fine — every voice can route to the
    // internal synth via voice_outputs. Pre-#36 the engine required
    // at least one external MIDI port; that constraint is gone now
    // that per-voice routing is the source of truth.

    // Issue #14: if the user selected external MIDI ports but all
    // voices route to the internal synth, the per-voice routing
    // table (`VoiceOutputTarget::default() = Synth`) silently swallows
    // their harmonies. Surface a one-time warning so this isn't a
    // mystery support thread — both to the desktop log and to the
    // frontend so the UI can show it.
    {
        let voice_outputs_snapshot = state
            .voice_outputs
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        if let Some(warning) =
            detect_no_external_output_warning(&voice_outputs_snapshot, &output_indices)
        {
            eprintln!("[start_routing] {}", warning);
            let _ = app_handle.emit("routing-warning", warning);
        }
    }

    // Share the engine across the router thread and command handlers.
    // Without this clone, the router would run on its own private engine
    // instance and would never see param changes (set_key, set_auto_key,
    // ...) made via Tauri commands during a routing session.
    let engine = Arc::clone(&state.engine);

    let routing_mode = { *state.routing_mode.lock().map_err(|e| e.to_string())? };

    // Capture guitar config for the router thread
    let is_guitar = input_idx == GUITAR_AUDIO_SENTINEL;
    let guitar_device = {
        state
            .guitar_device
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
    };
    let guitar_channel = { *state.guitar_channel.lock().map_err(|e| e.to_string())? };
    let guitar_config = {
        state
            .guitar_config
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .unwrap_or_default()
    };
    // Clone the Arc so the audio thread can re-read the live config every
    // block — lets the debug window's edits take effect without a restart.
    let guitar_config_shared = Arc::clone(&state.guitar_config);

    // Snapshot the calibration profile at routing-start so the bridge
    // applies it once at GuitarInput construction. Mid-session changes
    // (load_calibration_profile from the UI, or a save_calibration_profile
    // after a sweep) DO propagate live now via state.live_guitar_pipeline
    // — the bridge publishes its pipeline Arc on construction, and the
    // calibration commands try-lock + push the new profile. A source
    // toggle (MIDI → guitar) without restarting routing still requires
    // a restart because we'd need to construct a new bridge. The snapshot
    // here is the construction-time profile; runtime updates flow through
    // the live_guitar_pipeline slot.
    let live_pipeline_handle = if is_guitar {
        Some(Arc::clone(&state.live_guitar_pipeline))
    } else {
        None
    };
    let calibration_profile_snapshot = if is_guitar {
        Some(
            state
                .calibration_profile
                .lock()
                .map_err(|e| e.to_string())?
                .clone(),
        )
    } else {
        None
    };

    // Shared state for note updates
    let input_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let harmony_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let borrowed_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let chord_name = Arc::new(Mutex::new(String::new()));
    let stop_signal = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Stop any previous router thread before starting a new one
    if let Ok(mut prev_stop) = state.stop_signal.lock() {
        if let Some(prev) = prev_stop.take() {
            prev.store(true, Ordering::SeqCst);
        }
    }
    // Clear the live guitar pipeline slot unconditionally before
    // starting the new router. If the previous routing was guitar
    // and the new one is MIDI, the slot would otherwise retain an
    // Arc to a dead pipeline — subsequent calibration commands
    // would silently mutate the zombie. Brutal-critic round 3
    // CRITICAL. The slot will be re-populated by GuitarBridge::new
    // below when the new bridge succeeds.
    if let Ok(mut slot) = state.live_guitar_pipeline.lock() {
        *slot = None;
    }

    // Store the new stop signal so stop_routing can use it
    if let Ok(mut sig) = state.stop_signal.lock() {
        *sig = Some(Arc::clone(&stop_signal));
    }

    state.is_running.store(true, Ordering::SeqCst);

    // Create the MIDI-input channel up front so we can share the Sender with
    // inject_note_on / inject_note_off commands (virtual input from the UI).
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    {
        let mut slot = state.router_tx.lock().map_err(|e| e.to_string())?;
        *slot = Some(tx.clone());
    }

    // Clone Arcs for the router thread
    let in_notes = Arc::clone(&input_notes);
    let harm_notes = Arc::clone(&harmony_notes);
    let borr_notes = Arc::clone(&borrowed_notes);
    let ch_name = Arc::clone(&chord_name);
    let stop = Arc::clone(&stop_signal);
    let output_indices_clone = output_indices.clone();

    let detune = Arc::clone(&state.detune_cents);
    let panic_flag = Arc::clone(&state.panic_pending);
    let route_changes = Arc::clone(&state.route_changes);
    let performance_reset_revision = Arc::clone(&state.performance_reset_revision);
    // Reset before spawning so flags set from a prior session don't fire.
    panic_flag.store(false, Ordering::SeqCst);
    *route_changes
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = PendingRouteChanges::default();

    // Clone the synth event sender so the router thread can push note
    // events into the built-in synth alongside external MIDI output.
    let synth_tx = state.synth_tx.clone();

    // Share the per-voice output routing table with the router thread.
    // Lock-read at each note dispatch to honor live UI changes.
    let voice_outputs = Arc::clone(&state.voice_outputs);

    // Share the transport clock with the router thread so it can push
    // current beat-phase to the engine each iteration. Counterpoint
    // Species 2-4 read this; without a fresh phase they fall back to
    // Species 1 behavior.
    let transport = Arc::clone(&state.transport);

    // #91 commit A: share the Companion orchestrator with the router
    // thread. Defaults to enabled=false with zero Lanes — tick()
    // short-circuits and produces no DispatchOps until Lanes
    // register and the master switch flips.
    let companion = Arc::clone(&state.companion);
    let looper = Arc::clone(&state.looper);
    let slide_config = Arc::clone(&state.slide_config);
    let midi_slide_telemetry = Arc::clone(&state.midi_slide_telemetry);

    // Spawn router thread
    thread::spawn(move || {
        if let Err(e) = run_tauri_router(
            input_idx,
            &output_indices_clone,
            engine,
            routing_mode,
            is_guitar,
            guitar_device,
            guitar_channel,
            guitar_config,
            guitar_config_shared,
            calibration_profile_snapshot,
            live_pipeline_handle,
            tx,
            rx,
            in_notes,
            harm_notes,
            borr_notes,
            ch_name,
            stop,
            app_handle,
            detune,
            panic_flag,
            route_changes,
            performance_reset_revision,
            synth_tx,
            voice_outputs,
            transport,
            companion,
            looper,
            slide_config,
            midi_slide_telemetry,
        ) {
            eprintln!("[tauri-router] Error: {}", e);
        }
    });

    Ok(())
}

/// Stops the currently active MIDI routing.
#[tauri::command]
pub fn stop_routing(state: State<AppState>) -> Result<(), String> {
    if !state.is_running.load(Ordering::SeqCst) {
        return Err("Routing is not active".to_string());
    }

    // Signal the router thread to stop
    if let Ok(mut sig) = state.stop_signal.lock() {
        if let Some(stop) = sig.take() {
            stop.store(true, Ordering::SeqCst);
        }
    }

    // Drop the injection sender so further inject_note_on/off calls fail fast
    if let Ok(mut tx_slot) = state.router_tx.lock() {
        *tx_slot = None;
    }

    state.is_running.store(false, Ordering::SeqCst);

    // Drop the live pipeline handle so subsequent calibration commands
    // know there's nothing running to hot-swap into. The cpal stream
    // owned by GuitarBridge is dropped via the router-thread teardown.
    if let Ok(mut slot) = state.live_guitar_pipeline.lock() {
        *slot = None;
    }

    Ok(())
}

// ============================================================================
// Router thread implementation
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn run_tauri_router(
    input_port: usize,
    output_ports: &[usize],
    engine: Arc<Mutex<HarmonyEngine>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    is_guitar: bool,
    guitar_device: String,
    guitar_channel: usize,
    guitar_config: GuitarInputConfig,
    guitar_config_shared: Arc<Mutex<Option<GuitarInputConfig>>>,
    calibration_profile: Option<contrapunk::audio::guitar::GuitarCalibrationProfile>,
    live_pipeline_handle: Option<
        Arc<Mutex<Option<Arc<Mutex<contrapunk::audio::guitar_input::GuitarInput>>>>>,
    >,
    tx: mpsc::Sender<Vec<u8>>,
    rx: mpsc::Receiver<Vec<u8>>,
    input_notes: Arc<Mutex<HashSet<u8>>>,
    harmony_notes: Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: Arc<Mutex<HashSet<u8>>>,
    chord_name: Arc<Mutex<String>>,
    stop_signal: Arc<std::sync::atomic::AtomicBool>,
    app_handle: AppHandle,
    detune_cents: Arc<AtomicI32>,
    panic_pending: Arc<std::sync::atomic::AtomicBool>,
    route_changes: Arc<Mutex<PendingRouteChanges>>,
    performance_reset_revision: Arc<AtomicU64>,
    synth_tx: SynthEventSender,
    voice_outputs: Arc<Mutex<VoiceOutputRoutes>>,
    transport: Arc<Transport>,
    companion: Arc<Mutex<crate::companion::Companion>>,
    looper: Arc<Mutex<contrapunk_companion::LooperLane>>,
    slide_config: Arc<Mutex<SlideConfig>>,
    midi_slide_telemetry: Arc<SlideTelemetry>,
) -> anyhow::Result<()> {
    // Connect to either Guitar Audio bridge, physical MIDI input, or
    // nothing at all (Computer Keyboard virtual input — notes are pushed
    // by inject_note_on/off commands via the shared router_tx).
    let _midi_conn;
    let _guitar_bridge;
    let is_virtual = input_port == VIRTUAL_COMPUTER_KEYBOARD || input_port == VIRTUAL_TONE_SOURCE;

    // Signal channel for guitar UI feedback (only used in guitar mode)
    let (signal_tx, signal_rx) = mpsc::channel::<crate::guitar_bridge::GuitarSignalInfo>();

    if is_guitar {
        // Guitar Audio mode: spawn cpal capture -> DSP -> same tx channel
        let bridge = GuitarBridge::new(
            &guitar_device,
            guitar_channel,
            guitar_config,
            guitar_config_shared,
            calibration_profile,
            live_pipeline_handle,
            tx,
            Some(signal_tx),
        )
        .map_err(|e| anyhow::anyhow!("Guitar bridge error: {}", e))?;
        _guitar_bridge = Some(bridge);
        _midi_conn = None;
    } else if is_virtual {
        // Computer Keyboard / Tone mode: no physical connection. The tx is kept
        // alive by the AppState clone so inject_note_on/off can push.
        // Drop the local tx copy; AppState keeps the channel open.
        drop(tx);
        _midi_conn = None;
        _guitar_bridge = None;
    } else {
        // Physical MIDI mode
        _midi_conn = Some(connect_input(input_port, tx)?);
        _guitar_bridge = None;
    };

    // Create output router
    let mut output_router = OutputRouter::new(output_ports)?;

    // voice_count is user-controlled via `set_voice_count`. With
    // per-voice routing, voices in excess of the connected MIDI ports
    // route to the built-in synth — there's no reason to clamp the
    // engine's voice_count to the output count at routing start. (Doing
    // so silently overrode the UI's voice picker — see the wave of
    // "I picked soprano in a 4-voice setup but the engine had only
    // 2 voices" reports.)

    // Per-lane counters preserve overlapping voices at the same pitch.
    // They stay separate from generic harmony so Live Lines can render
    // each musical source without subtracting ambiguous pitch unions.
    let canon_notes: Arc<Mutex<NoteCounts>> = Arc::new(Mutex::new(HashMap::new()));
    let counterpoint_notes: Arc<Mutex<NoteCounts>> = Arc::new(Mutex::new(HashMap::new()));
    let mut companion_output_notes = RoutedNoteCounts::new();
    let mut loop_output_notes = RoutedNoteCounts::new();
    let mut sustain_owners: SustainOwners = HashSet::new();

    // Loop replay gets independent harmony, lane, phrase, and note-state
    // ownership while sharing the sample-derived transport and current config.
    let loop_engine = Arc::new(Mutex::new(
        engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .fork_clean_runtime(),
    ));
    let initial_companion_state = companion
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .save();
    let mut loop_companion =
        crate::state::new_arrangement_companion(Arc::clone(&transport), Arc::clone(&loop_engine));
    loop_companion
        .restore(initial_companion_state.clone())
        .map_err(anyhow::Error::msg)?;
    let mut applied_companion_state = initial_companion_state;
    let loop_input_notes = Arc::new(Mutex::new(HashSet::new()));
    let loop_harmony_notes: Arc<Mutex<HashSet<u8>>> = Arc::new(Mutex::new(HashSet::new()));
    let loop_canon_notes: Arc<Mutex<NoteCounts>> = Arc::new(Mutex::new(HashMap::new()));
    let loop_counterpoint_notes = Arc::new(Mutex::new(HashMap::new()));
    let mut loop_source_frames: LoopSourceFrames = HashMap::new();
    let mut last_transport_revision = transport.discontinuity_revision();
    let mut transport_was_running = transport.is_running();
    let mut companion_tick_scheduler = BeatTickScheduler::new(&transport);
    let mut applied_performance_reset_revision = performance_reset_revision.load(Ordering::Acquire);

    // Event emission timer (~30fps)
    let mut last_emit = Instant::now();
    let emit_interval = Duration::from_millis(33);

    // Detune: track the previous value so we only send pitch bend on change.
    let mut prev_detune_cents: i32 = detune_cents.load(Ordering::Relaxed);
    let mut midi_slides = MidiSlideRuntime::new(prev_detune_cents, midi_slide_telemetry);
    let mut last_cc_panic_broadcast: Option<Instant> = None;

    // Main routing loop
    loop {
        if stop_signal.load(Ordering::SeqCst) {
            break;
        }

        // A destination edit applies at a clean note boundary. A single-route
        // edit releases only that route; the global synth override and full
        // performance reset intentionally drain everything.
        let pending_routes = route_changes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if !pending_routes.all() {
            let num_ports = output_router.connection_count();
            for route in pending_routes.routes() {
                drain_route_outputs(
                    route,
                    &mut companion_output_notes,
                    &mut loop_output_notes,
                    num_ports,
                    &synth_tx,
                    &mut output_router,
                    &mut midi_slides,
                );
            }
        }
        if pending_routes.all() {
            let num_ports = output_router.connection_count();
            cleanup_loop_outputs(
                &mut companion_output_notes,
                &mut loop_output_notes,
                &mut sustain_owners,
                num_ports,
                &synth_tx,
                &mut output_router,
                &mut midi_slides,
            );
            clear_all_sustain(
                &mut sustain_owners,
                num_ports,
                &synth_tx,
                &mut output_router,
                &mut midi_slides,
            );
            drain_routed_outputs(
                &mut companion_output_notes,
                num_ports,
                &synth_tx,
                &mut output_router,
                &mut midi_slides,
            );
            midi_slides.clear(&mut output_router);
            loop_output_notes.clear();
            let _ = drain_all_tracked_notes(
                &input_notes,
                &harmony_notes,
                &borrowed_notes,
                &canon_notes,
                &counterpoint_notes,
            );
            chord_name
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear();
            loop_source_frames.clear();
            loop_input_notes
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear();
            loop_harmony_notes
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear();
            loop_canon_notes
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear();
            loop_counterpoint_notes
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear();
            engine
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear_active_notes();
            companion
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .reset_runtime();
            loop_engine
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear_active_notes();
            loop_companion.reset_runtime();
        }

        let reset_revision = performance_reset_revision.load(Ordering::Acquire);
        if reset_revision != applied_performance_reset_revision {
            let companion_state = companion
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .save();
            *loop_engine
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = engine
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .fork_clean_runtime();
            loop_companion = crate::state::new_arrangement_companion(
                Arc::clone(&transport),
                Arc::clone(&loop_engine),
            );
            loop_companion
                .restore(companion_state.clone())
                .map_err(anyhow::Error::msg)?;
            applied_companion_state = companion_state;
            companion_tick_scheduler.reset(&transport);
            applied_performance_reset_revision = reset_revision;
        }

        // Push current beat-phase from the transport clock to the engine.
        // Counterpoint Species 2-4 read this on every NoteOn to decide
        // passing tones / suspensions / strong-vs-weak-beat behavior.
        // Without a fresh phase, the engine sees None and silently falls
        // back to Species 1 regardless of which Species the user picked.
        // When transport is stopped we explicitly push None so Species
        // 2-4 fall back to Species 1 (per the engine doc on
        // `counterpoint_beat_phase: Option<f64>`).
        {
            let phase = if transport.is_running() {
                Some(transport.beat_position())
            } else {
                None
            };
            let mut eng = engine.lock().unwrap_or_else(|e| e.into_inner());
            eng.set_counterpoint_beat_phase(phase);
        }

        // Handle transport/control discontinuities and arrangement changes
        // before any loop event or live Companion tick can attack notes.
        let transport_revision = transport.discontinuity_revision();
        let transport_running = transport.is_running();
        let mut cleanup_loop = {
            let mut loop_state = looper.lock().unwrap_or_else(|e| e.into_inner());
            if transport_revision != last_transport_revision {
                if !loop_state.take_accepted_discontinuity(transport_revision) {
                    loop_state.transport_discontinuity();
                }
                last_transport_revision = transport_revision;
            }
            if transport_was_running && !transport_running {
                loop_state.transport_discontinuity();
            }
            loop_state.take_cleanup_request()
        };
        transport_was_running = transport_running;

        let live_companion_state = companion.lock().unwrap_or_else(|e| e.into_inner()).save();
        let harmony_config_changed = {
            let live = engine.lock().unwrap_or_else(|e| e.into_inner());
            let mut replay = loop_engine.lock().unwrap_or_else(|e| e.into_inner());
            // Spread changes are next-note parameters: keep existing loop-owned
            // frames intact and let subsequent replay NoteOns use the new value.
            replay.set_octave_intensity(live.octave_intensity());
            replay.set_octave_mode(live.octave_mode());
            !replay.has_same_configuration(&live)
        };
        let companion_config_changed = live_companion_state != applied_companion_state;
        let rebuild_loop_runtime = harmony_config_changed || companion_config_changed;
        cleanup_loop |= rebuild_loop_runtime;

        if cleanup_loop {
            cleanup_loop_outputs(
                &mut companion_output_notes,
                &mut loop_output_notes,
                &mut sustain_owners,
                output_router.connection_count(),
                &synth_tx,
                &mut output_router,
                &mut midi_slides,
            );
            loop_source_frames.clear();
            loop_input_notes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clear();
            loop_harmony_notes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clear();
            loop_canon_notes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clear();
            loop_counterpoint_notes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clear();
        }

        if rebuild_loop_runtime {
            *loop_engine.lock().unwrap_or_else(|e| e.into_inner()) = engine
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .fork_clean_runtime();
            loop_companion = crate::state::new_arrangement_companion(
                Arc::clone(&transport),
                Arc::clone(&loop_engine),
            );
            loop_companion
                .restore(live_companion_state.clone())
                .map_err(anyhow::Error::msg)?;
            applied_companion_state = live_companion_state;
        } else if cleanup_loop {
            loop_engine
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clear_active_notes();
            loop_companion.reset_runtime();
        }

        // Run loop replay plus both Companion runtimes on one shared beat grid.
        // Interleaving each slot keeps semantic dispatch order independent of
        // audio block size or router polling batches.
        let beats_per_bar = f64::from(transport.time_signature().0);
        for slot in companion_tick_scheduler.due_slots(&transport) {
            let beat = BeatTickScheduler::beat(slot);
            let phase = transport_running.then(|| beat.rem_euclid(beats_per_bar));
            engine
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .set_counterpoint_beat_phase(phase);
            loop_engine
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .set_counterpoint_beat_phase(phase);

            let tagged = loop_companion.tick_tagged_at(beat, &loop_engine);
            dispatch_companion_ops(
                &tagged,
                output_router.connection_count(),
                &synth_tx,
                &mut output_router,
                &mut midi_slides,
                &voice_outputs,
                &loop_harmony_notes,
                &loop_canon_notes,
                &loop_counterpoint_notes,
                &mut companion_output_notes,
                &mut loop_output_notes,
                InputOrigin::Loop,
                &slide_config,
            );

            let replay = if transport_running {
                looper.lock().unwrap_or_else(|e| e.into_inner()).tick(beat)
            } else {
                Vec::new()
            };
            for replay_event in replay {
                let event_beat = replay_event.scheduled_beat().unwrap_or(beat);
                loop_engine
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .set_counterpoint_beat_phase(
                        transport_running.then(|| event_beat.rem_euclid(beats_per_bar)),
                    );
                let bytes = loop_event_bytes(replay_event.event);
                let mut suppress_default = false;
                if let Some(input) = midi_bytes_to_input_event(&bytes) {
                    let (tagged, suppress) =
                        loop_companion.on_input_tagged_at(input, event_beat, &loop_engine);
                    dispatch_companion_ops(
                        &tagged,
                        output_router.connection_count(),
                        &synth_tx,
                        &mut output_router,
                        &mut midi_slides,
                        &voice_outputs,
                        &loop_harmony_notes,
                        &loop_canon_notes,
                        &loop_counterpoint_notes,
                        &mut companion_output_notes,
                        &mut loop_output_notes,
                        InputOrigin::Loop,
                        &slide_config,
                    );
                    suppress_default = suppress;
                }
                match replay_event.event {
                    LoopMidiEvent::Cc64 { value, channel } => set_sustain_ownership(
                        InputOrigin::Loop,
                        channel,
                        value >= 64,
                        &mut sustain_owners,
                        &voice_outputs,
                        output_router.connection_count(),
                        &synth_tx,
                        &mut output_router,
                        &mut midi_slides,
                    ),
                    _ if !suppress_default => process_loop_midi_event(
                        replay_event.event,
                        &loop_engine,
                        &mut output_router,
                        &mut midi_slides,
                        &loop_input_notes,
                        &loop_harmony_notes,
                        &voice_outputs,
                        &slide_config,
                        &mut companion_output_notes,
                        &mut loop_output_notes,
                        &mut loop_source_frames,
                        &synth_tx,
                    ),
                    _ => {}
                }
            }

            let live_tagged = {
                let mut live = companion.lock().unwrap_or_else(|e| e.into_inner());
                live.tick_tagged_at(beat, &engine)
            };
            dispatch_companion_ops(
                &live_tagged,
                output_router.connection_count(),
                &synth_tx,
                &mut output_router,
                &mut midi_slides,
                &voice_outputs,
                &harmony_notes,
                &canon_notes,
                &counterpoint_notes,
                &mut companion_output_notes,
                &mut loop_output_notes,
                InputOrigin::Live,
                &slide_config,
            );
        }

        // Incoming live MIDI belongs to the physical transport observation,
        // not the last quantized scheduler slot.
        engine
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .set_counterpoint_beat_phase(transport_running.then(|| transport.beat_position()));

        // Beat-aligned chord retrigger. When pattern is enabled and the
        // transport is running, detect cell-boundary crossings and
        // retrigger the currently-sounding harmony on each cell-on
        // boundary (Live mode). Skip silently when pattern off or
        // transport stopped — fall through to today's real-time path.
        //
        // Skip entirely when `panic_pending` is set: this iteration
        // belongs to the reharmonize cycle below, which will replay
        // held inputs and dispatch a clean diff. Firing the pattern
        // tick first would attack stale `harmony_notes` and produce an
        // audible click as reharm immediately releases the displaced
        // voices.
        // Reharmonize on parameter change. Any engine-config setter that
        // could change the harmony output sets panic_pending and stashes
        // the previously-held input MIDI numbers in the engine's
        // `pending_reharm_inputs`. We replay each of those inputs under
        // the new parameters, compute a diff against the previously-
        // sounding harmony set, and dispatch only the difference:
        // NoteOff for harmony notes that drop out, NoteOn for newly-
        // needed ones. The user's held input note never gets
        // interrupted — knob sweeps produce a smooth musical morph.
        if panic_pending.swap(false, Ordering::SeqCst) {
            // Snapshot what's currently sounding (harmony + borrowed only —
            // input notes belong to the user, leave them alone).
            let old_harmonies: HashSet<u8> = {
                let h = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
                let b = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
                h.iter().chain(b.iter()).copied().collect()
            };

            // Drain held inputs from the engine and replay them under the
            // new parameters. Each replay populates `active_notes` and
            // updates `last_port_map` for the per-voice routing below.
            let mut new_harmonies: HashSet<u8> = HashSet::new();
            let mut per_input: Vec<(u8, u8, Vec<(u8, f32)>, Vec<usize>)> = Vec::new();
            {
                let mut eng = engine.lock().unwrap_or_else(|e| e.into_inner());
                let inputs = eng.take_owned_reharm_inputs_with_velocity();
                for (source, midi, velocity) in inputs {
                    if let Ok(input_note) = Note::try_from(midi) {
                        let result =
                            eng.harmonize_note_on_owned_with_velocity(input_note, source, velocity);
                        let port_map = eng.last_port_map().to_vec();
                        let tuning = eng.tune_harmony(&result).ok();
                        let voices: Vec<(u8, f32)> = result
                            .iter()
                            .enumerate()
                            .map(|(index, note)| {
                                let midi = u8::from(*note);
                                let frequency = tuning
                                    .as_ref()
                                    .and_then(|frame| frame.as_slice().get(index))
                                    .map(|pitch| pitch.frequency_hz as f32)
                                    .unwrap_or_else(|| standard_frequency(midi));
                                (midi, frequency)
                            })
                            .collect();
                        for &(midi, _) in voices.iter().skip(1) {
                            new_harmonies.insert(midi);
                        }
                        per_input.push((source, velocity, voices, port_map));
                    }
                }
            }

            let mut to_release: Vec<u8> =
                old_harmonies.difference(&new_harmonies).copied().collect();
            to_release.sort_unstable();
            let to_attack: HashSet<u8> =
                new_harmonies.difference(&old_harmonies).copied().collect();

            // Release only the exact destinations that owned each stale note.
            let num_ports = output_router.connection_count();
            for n in &to_release {
                loop_output_notes.remove_harmony_note(*n);
                for key in companion_output_notes.remove_harmony_note(*n) {
                    release_routed_key(
                        key,
                        num_ports,
                        &synth_tx,
                        &mut output_router,
                        &mut midi_slides,
                    );
                }
            }

            // Send NoteOn for newly-attacked notes, routed per voice via
            // each replay's port map and the live voice_outputs table.
            let voice_targets = voice_outputs
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            let slide_snapshot = *slide_config
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            for (channel, velocity, voices, port_map) in &per_input {
                // Skip index 0 — that's the user's input note, already
                // sounding from when they pressed the key.
                for (i, &(n, frequency_hz)) in voices.iter().enumerate().skip(1) {
                    if !to_attack.contains(&n) {
                        continue; // already sounding from before
                    }
                    let slot = port_map.get(i).copied().unwrap_or(i);
                    let route = VoiceRouteId::Harmony { slot: slot as u8 };
                    let target = voice_targets.get(route);
                    if count_origin_note_on(
                        InputOrigin::Live,
                        &mut companion_output_notes,
                        &mut loop_output_notes,
                        route,
                        routed_note_key(target, *channel, n, MIX_HARMONY, slot as u8),
                    ) {
                        dispatch_voice(
                            target,
                            MIX_HARMONY,
                            *channel,
                            VoiceDispatch::NoteOn {
                                note: n,
                                frequency_hz,
                                velocity: *velocity,
                                slide_slot: SlideSlot::new(SlideRole::Harmony, slot as u8),
                                slide: slide_snapshot
                                    .resolve(SlideSlot::new(SlideRole::Harmony, slot as u8))
                                    .unwrap_or_default(),
                            },
                            num_ports,
                            &synth_tx,
                            &mut output_router,
                            &mut midi_slides,
                        );
                    }
                }
            }

            // Replace UI tracking sets with the new harmony state. Input
            // notes stay as-is — user's still holding them.
            {
                let mut h = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
                *h = new_harmonies;
            }
            {
                let mut b = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
                b.clear();
            }

            // Rebuild routing-aware tracking from the replay results.
            // Each input's harmonies (skipping i=0, the input itself)
            // get re-keyed with their freshly-resolved per-voice
            // target. Pattern attacks/releases on subsequent ticks
        }

        // Apply detune as MIDI pitch bend when the value changes.
        let current_detune = detune_cents.load(Ordering::Relaxed);
        if current_detune != prev_detune_cents {
            prev_detune_cents = current_detune;
            midi_slides.set_detune(current_detune, &mut output_router);
        }
        midi_slides.tick(&mut output_router);

        // Process MIDI messages
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(message) => {
                // Capture normalized Live MIDI before Companion or harmony.
                if let Some(event) = midi_bytes_to_loop_event(&message, InputOrigin::Live) {
                    looper
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .capture(event, transport.total_beats());
                }

                // Intercept Control Change messages (status 0xB0-0xBF) and
                // forward every CC to the frontend as "knob-cc-raw". The UI
                // owns the CC → Performance-view-knob mapping (MIDI Learn,
                // persisted in localStorage). MPK Mini's CC 70-77 baseline
                // is seeded as the default preset on first run.
                if message.len() >= 3 && (message[0] & 0xF0) == 0xB0 {
                    let cc_number = message[1];
                    let cc_value = message[2];

                    // Issue #90 fast-path: CC 120/123 = All Sound/Notes Off.
                    // Drain every tracked note and send
                    // NoteOff downstream so the user can recover from
                    // dropped Note-Offs / MPE channel rotation / device
                    // disconnect mid-phrase without restarting routing.
                    // The full reconcile-against-engine.active_notes
                    // story is deferred — this gives users a one-button
                    // escape today.
                    if matches!(cc_number, 120 | 123) {
                        looper
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .transport_discontinuity();
                        let num_ports = output_router.connection_count();
                        cleanup_loop_outputs(
                            &mut companion_output_notes,
                            &mut loop_output_notes,
                            &mut sustain_owners,
                            num_ports,
                            &synth_tx,
                            &mut output_router,
                            &mut midi_slides,
                        );
                        loop_source_frames.clear();
                        loop_engine
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .clear_active_notes();
                        loop_companion.reset_runtime();
                        clear_all_sustain(
                            &mut sustain_owners,
                            num_ports,
                            &synth_tx,
                            &mut output_router,
                            &mut midi_slides,
                        );
                        drain_routed_outputs(
                            &mut companion_output_notes,
                            num_ports,
                            &synth_tx,
                            &mut output_router,
                            &mut midi_slides,
                        );
                        loop_output_notes.clear();
                        let _ = drain_all_tracked_notes(
                            &input_notes,
                            &harmony_notes,
                            &borrowed_notes,
                            &canon_notes,
                            &counterpoint_notes,
                        );
                        midi_slides.clear(&mut output_router);
                        let now = Instant::now();
                        let should_broadcast =
                            should_rebroadcast_cc_panic(last_cc_panic_broadcast, now);
                        if should_broadcast {
                            send_all_notes_off(num_ports, &synth_tx, &mut output_router);
                            last_cc_panic_broadcast = Some(now);
                            eprintln!("[router] CC {cc_number} panic: cleared all tracked notes");
                        }
                        engine
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .clear_active_notes();
                        companion
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .reset_runtime();
                        // Continue to also forward to UI below so the
                        // Performance view's CC mapping still sees it.
                    }

                    match cc_number {
                        1 => {
                            let _ = synth_tx.mod_wheel(cc_value as f32 / 127.0);
                        }
                        11 => {
                            let _ = synth_tx.expression(cc_value as f32 / 127.0);
                        }
                        _ => {}
                    }

                    if cc_number == 64 {
                        set_sustain_ownership(
                            InputOrigin::Live,
                            message[0] & 0x0f,
                            cc_value >= 64,
                            &mut sustain_owners,
                            &voice_outputs,
                            output_router.connection_count(),
                            &synth_tx,
                            &mut output_router,
                            &mut midi_slides,
                        );
                    }

                    if let Some(ev) = midi_bytes_to_input_event(&message) {
                        let tagged = {
                            let mut c = companion.lock().unwrap_or_else(|e| e.into_inner());
                            c.on_input_tagged(ev, &engine).0
                        };
                        let num_ports = output_router.connection_count();
                        dispatch_companion_ops(
                            &tagged,
                            num_ports,
                            &synth_tx,
                            &mut output_router,
                            &mut midi_slides,
                            &voice_outputs,
                            &harmony_notes,
                            &canon_notes,
                            &counterpoint_notes,
                            &mut companion_output_notes,
                            &mut loop_output_notes,
                            InputOrigin::Live,
                            &slide_config,
                        );
                    }

                    let value = (cc_value as f32) / 127.0;
                    let _ = app_handle.emit(
                        "knob-cc-raw",
                        KnobCcRawPayload {
                            cc: cc_number,
                            value,
                        },
                    );
                    // Don't fall through — CCs are not notes. Forwarding to
                    // process_midi_message would route them to send_to_first
                    // which is wrong for control data.
                } else {
                    // #91 commit B: ask the Companion's input-pipeline
                    // Lanes to inspect this event first. If any Lane
                    // returns `suppress_default = true`, skip the
                    // existing harmony dispatch — the Lane has taken
                    // over for this event. Returned ops are dispatched
                    // through the same translator as tick() ops.
                    let mut suppress_default = false;
                    if let Some(ev) = midi_bytes_to_input_event(&message) {
                        let (tagged, sup) = {
                            let mut c = companion.lock().unwrap_or_else(|e| e.into_inner());
                            c.on_input_tagged(ev, &engine)
                        };
                        let num_ports = output_router.connection_count();
                        dispatch_companion_ops(
                            &tagged,
                            num_ports,
                            &synth_tx,
                            &mut output_router,
                            &mut midi_slides,
                            &voice_outputs,
                            &harmony_notes,
                            &canon_notes,
                            &counterpoint_notes,
                            &mut companion_output_notes,
                            &mut loop_output_notes,
                            InputOrigin::Live,
                            &slide_config,
                        );
                        suppress_default = sup;
                    }
                    if !suppress_default {
                        process_midi_message(
                            &message,
                            InputOrigin::Live,
                            &engine,
                            &mut output_router,
                            &mut midi_slides,
                            &input_notes,
                            &harmony_notes,
                            &borrowed_notes,
                            &chord_name,
                            routing_mode,
                            &synth_tx,
                            &voice_outputs,
                            &slide_config,
                            &mut companion_output_notes,
                            &mut loop_output_notes,
                        );
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("[tauri-router] Input channel disconnected");
                break;
            }
        }

        // Emit note-update event at ~30fps
        if last_emit.elapsed() >= emit_interval {
            last_emit = Instant::now();
            // Recover from poisoned mutexes instead of panicking the router
            // thread. A poisoned lock means a thread panicked while holding
            // it — the data is likely still usable, and silently crashing
            // the emit loop leaves the UI stuck with no user-visible error.
            let (last_borrowed_from, current_key) = {
                let eng = engine.lock().unwrap_or_else(|e| e.into_inner());
                (
                    eng.last_borrowed_from()
                        .map(|m| format!("{}", m))
                        .unwrap_or_default(),
                    format!("{}", eng.key()),
                )
            };
            let payload = {
                let in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
                let harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
                let borr_notes = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
                let canon = canon_notes.lock().unwrap_or_else(|e| e.into_inner());
                let cp = counterpoint_notes.lock().unwrap_or_else(|e| e.into_inner());
                let ch_name = chord_name.lock().unwrap_or_else(|e| e.into_inner());
                let phrase = companion
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .phrase_snapshot();
                build_note_update_payload(
                    &in_notes,
                    &harm_notes,
                    &borr_notes,
                    ch_name.clone(),
                    last_borrowed_from,
                    current_key,
                    &canon,
                    &cp,
                    phrase,
                )
            };
            let _ = app_handle.emit("note-update", payload);

            // Emit guitar signal info for UI (drain latest from channel)
            if is_guitar {
                let mut latest_signal = None;
                while let Ok(sig) = signal_rx.try_recv() {
                    latest_signal = Some(sig);
                }
                if let Some(sig) = latest_signal {
                    let _ = app_handle.emit("guitar-signal", build_guitar_signal_payload(sig));
                }
            }
        }
    }

    // Release downstream sound while the router and its output handles
    // are still alive. Clearing bookkeeping alone would leave external
    // instruments and the built-in synth ringing after Stop.
    looper
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .transport_discontinuity();
    let num_ports = output_router.connection_count();
    cleanup_loop_outputs(
        &mut companion_output_notes,
        &mut loop_output_notes,
        &mut sustain_owners,
        num_ports,
        &synth_tx,
        &mut output_router,
        &mut midi_slides,
    );
    clear_all_sustain(
        &mut sustain_owners,
        num_ports,
        &synth_tx,
        &mut output_router,
        &mut midi_slides,
    );
    drain_routed_outputs(
        &mut companion_output_notes,
        num_ports,
        &synth_tx,
        &mut output_router,
        &mut midi_slides,
    );
    loop_output_notes.clear();
    let _ = drain_all_tracked_notes(
        &input_notes,
        &harmony_notes,
        &borrowed_notes,
        &canon_notes,
        &counterpoint_notes,
    );
    midi_slides.clear(&mut output_router);
    send_all_notes_off(num_ports, &synth_tx, &mut output_router);

    engine
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear_active_notes();
    companion
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .reset_runtime();
    loop_engine
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear_active_notes();
    loop_companion.reset_runtime();

    // Clear note state on exit. Per-lane sets (canon_notes /
    // counterpoint_notes) MUST also be cleared — otherwise stale
    // gold/lime piano colors persist across stop/start cycles
    // until a NoteOff for those exact pitches arrives via the
    // next session.
    {
        let mut notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }
    {
        let mut notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }
    {
        let mut notes = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }
    {
        let mut notes = canon_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }
    {
        let mut notes = counterpoint_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_midi_message(
    bytes: &[u8],
    origin: InputOrigin,
    engine: &Arc<Mutex<HarmonyEngine>>,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    synth_tx: &SynthEventSender,
    voice_outputs: &Arc<Mutex<VoiceOutputRoutes>>,
    slide_config: &Arc<Mutex<SlideConfig>>,
    output_notes: &mut RoutedNoteCounts,
    loop_output_notes: &mut RoutedNoteCounts,
) {
    let msg = match MidiMessage::try_from(bytes) {
        Ok(m) => m,
        Err(_) => {
            send_input_passthrough(bytes, voice_outputs, output);
            return;
        }
    };

    // Lock the shared engine for the duration of one MIDI message. Held
    // briefly enough that Tauri command handlers (set_key etc.) waiting
    // on the same Mutex pick it up between messages.
    let mut eng = engine.lock().unwrap_or_else(|e| e.into_inner());
    let eng: &mut HarmonyEngine = &mut eng;
    let slide_config = *slide_config
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    match msg {
        MidiMessage::NoteOn(channel, note, velocity) => {
            if velocity == Velocity::MIN {
                handle_note_off(
                    channel,
                    note,
                    velocity,
                    eng,
                    output,
                    midi_slides,
                    input_notes,
                    harmony_notes,
                    borrowed_notes,
                    chord_name,
                    routing_mode,
                    synth_tx,
                    voice_outputs,
                    origin,
                    output_notes,
                    loop_output_notes,
                );
            } else {
                handle_note_on(
                    channel,
                    note,
                    velocity,
                    eng,
                    output,
                    midi_slides,
                    input_notes,
                    harmony_notes,
                    borrowed_notes,
                    chord_name,
                    routing_mode,
                    synth_tx,
                    voice_outputs,
                    &slide_config,
                    origin,
                    output_notes,
                    loop_output_notes,
                );
            }
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            handle_note_off(
                channel,
                note,
                velocity,
                eng,
                output,
                midi_slides,
                input_notes,
                harmony_notes,
                borrowed_notes,
                chord_name,
                routing_mode,
                synth_tx,
                voice_outputs,
                origin,
                output_notes,
                loop_output_notes,
            );
        }
        MidiMessage::PitchBendChange(channel, bend) => {
            let bend_semitones = midi_input_bend_semitones(u16::from(bend));
            let _ = synth_tx.pitch_bend(bend_semitones * 100.0);
            let target = voice_outputs
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .get(VoiceRouteId::Input);
            if let VoiceOutputTarget::MidiPort { port } = target {
                midi_slides.set_input_bend(port, channel.index(), bend_semitones, output);
            }
        }
        MidiMessage::ChannelPressure(_, pressure) => {
            let _ = synth_tx.expression(u8::from(pressure) as f32 / 127.0);
            send_input_passthrough(bytes, voice_outputs, output);
        }
        _ => send_input_passthrough(bytes, voice_outputs, output),
    }
}

fn send_input_passthrough(
    bytes: &[u8],
    voice_outputs: &Arc<Mutex<VoiceOutputRoutes>>,
    output: &mut OutputRouter,
) {
    let target = voice_outputs
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(VoiceRouteId::Input);
    if let VoiceOutputTarget::MidiPort { port } = target {
        if let Err(error) = output.send_to_device_port(port, bytes) {
            eprintln!("[router] could not pass input MIDI to device {port}: {error}");
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn process_loop_midi_event(
    event: LoopMidiEvent,
    engine: &Arc<Mutex<HarmonyEngine>>,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    voice_outputs: &Arc<Mutex<VoiceOutputRoutes>>,
    slide_config: &Arc<Mutex<SlideConfig>>,
    output_notes: &mut RoutedNoteCounts,
    loop_output_notes: &mut RoutedNoteCounts,
    source_frames: &mut LoopSourceFrames,
    synth_tx: &SynthEventSender,
) {
    let (note, velocity, channel, note_on) = match event {
        LoopMidiEvent::NoteOn {
            note,
            velocity,
            channel,
        } if velocity > 0 => (note, velocity, channel, true),
        LoopMidiEvent::NoteOn { note, channel, .. }
        | LoopMidiEvent::NoteOff {
            note,
            channel,
            velocity: 0,
        } => (note, 0, channel, false),
        LoopMidiEvent::NoteOff {
            note,
            velocity,
            channel,
        } => (note, velocity, channel, false),
        LoopMidiEvent::Cc64 { .. } => return,
    };

    if note_on {
        let Ok(wmidi_note) = Note::try_from(note) else {
            return;
        };
        let (notes, tuning, port_map) = {
            let mut engine = engine
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let notes = engine.harmonize(wmidi_note);
            let tuning = engine.tune_harmony(&notes).ok();
            let port_map = engine.last_port_map().to_vec();
            (notes, tuning, port_map)
        };
        let targets = voice_outputs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let slide = *slide_config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let num_ports = output.connection_count();
        let mut owned = Vec::with_capacity(notes.len());
        for (index, generated) in notes.iter().copied().enumerate() {
            let voice_slot = port_map.get(index).copied().unwrap_or(index) as u8;
            let route = main_voice_route(index, voice_slot);
            let target = targets.get(route);
            let mix_group = if index == 0 { MIX_INPUT } else { MIX_HARMONY };
            let slide_slot = if index == 0 {
                SlideSlot::new(SlideRole::Input, 0)
            } else {
                SlideSlot::new(SlideRole::Harmony, voice_slot)
            };
            let generated = u8::from(generated);
            let key = routed_note_key(target, channel, generated, mix_group, voice_slot);
            if count_origin_note_on(
                InputOrigin::Loop,
                output_notes,
                loop_output_notes,
                route,
                key,
            ) {
                dispatch_voice(
                    target,
                    mix_group,
                    channel,
                    VoiceDispatch::NoteOn {
                        note: generated,
                        frequency_hz: tuning
                            .as_ref()
                            .and_then(|frame| frame.as_slice().get(index))
                            .map(|pitch| pitch.frequency_hz as f32)
                            .unwrap_or_else(|| standard_frequency(generated)),
                        velocity,
                        slide_slot,
                        slide: slide.resolve(slide_slot).unwrap_or_default(),
                    },
                    num_ports,
                    synth_tx,
                    output,
                    midi_slides,
                );
            }
            owned.push(LoopOwnedVoice {
                route,
                key,
                target,
                mix_group,
                channel,
                note: generated,
                slide_slot,
            });
        }
        source_frames
            .entry((channel, note))
            .or_default()
            .push_back(owned);
        input_notes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(note);
        let mut harmony = harmony_notes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        harmony.extend(notes.iter().skip(1).map(|note| u8::from(*note)));
        return;
    }

    let Some(frames) = source_frames.get_mut(&(channel, note)) else {
        return;
    };
    let Some(owned) = frames.pop_front() else {
        return;
    };
    if frames.is_empty() {
        source_frames.remove(&(channel, note));
    }
    let num_ports = output.connection_count();
    for voice in owned {
        if count_origin_note_off(
            InputOrigin::Loop,
            output_notes,
            loop_output_notes,
            voice.route,
            voice.key,
        ) {
            dispatch_voice(
                voice.target,
                voice.mix_group,
                voice.channel,
                VoiceDispatch::NoteOff {
                    note: voice.note,
                    velocity,
                    slide_slot: Some(voice.slide_slot),
                },
                num_ports,
                synth_tx,
                output,
                midi_slides,
            );
        }
        if voice.mix_group == MIX_HARMONY {
            harmony_notes
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&voice.note);
        }
    }
    input_notes
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&note);
}

#[allow(clippy::too_many_arguments)]
fn handle_note_on(
    channel: Channel,
    note: Note,
    velocity: Velocity,
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    chord_name: &Arc<Mutex<String>>,
    _routing_mode: contrapunk::harmony::RoutingMode,
    synth_tx: &SynthEventSender,
    voice_outputs: &Arc<Mutex<VoiceOutputRoutes>>,
    slide_config: &SlideConfig,
    origin: InputOrigin,
    output_notes: &mut RoutedNoteCounts,
    loop_output_notes: &mut RoutedNoteCounts,
) {
    let velocity_byte = u8::from(velocity);
    let notes = engine.harmonize_note_on_owned_with_velocity(note, channel.index(), velocity_byte);
    let tuning = engine.tune_harmony(&notes).ok();
    // Drain any harmonies the engine flagged for explicit release —
    // populated when an auto-key change wiped `active_notes` mid-flight.
    // These would otherwise stay sounding under the old key.
    let stale_releases = engine.take_pending_releases();
    let num_outputs = output.connection_count();

    // Send Note-Offs to the exact stale owners before emitting new ones.
    if !stale_releases.is_empty() {
        for &n in &stale_releases {
            let note = u8::from(n);
            loop_output_notes.remove_harmony_note(note);
            for key in output_notes.remove_harmony_note(note) {
                release_routed_key(key, num_outputs, synth_tx, output, midi_slides);
            }
        }
        let mut harm = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        let mut borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in &stale_releases {
            harm.remove(&u8::from(n));
            borr.remove(&u8::from(n));
        }
    }

    // Snapshot destinations once per event. The performed input has a stable
    // route of its own; generated harmony retains SATB arrangement slots.
    let voice_targets = voice_outputs
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let port_map: Vec<usize> = engine.last_port_map().to_vec();
    let target_for = |i: usize| -> VoiceOutputTarget {
        voice_targets.get(main_voice_route(
            i,
            port_map.get(i).copied().unwrap_or(i) as u8,
        ))
    };

    // Fan each voice via the unified dispatch helper. Synth and
    // external MIDI go in one loop; `dispatch_voice` handles the
    // target.match cases internally. Order: dispatch first so the
    // synth's audible NoteOn precedes the tracking-set updates that
    // chord-display reads from (chord display tolerates briefly
    // stale state better than humans tolerate audible latency).
    let channel_idx: u8 = channel.index();
    for (i, &n) in notes.iter().enumerate() {
        let arrangement_slot = port_map.get(i).copied().unwrap_or(i);
        let slide_slot = if i == 0 {
            SlideSlot::new(SlideRole::Input, 0)
        } else {
            SlideSlot::new(SlideRole::Harmony, arrangement_slot as u8)
        };
        let route = main_voice_route(i, arrangement_slot as u8);
        let target = target_for(i);
        let mix_group = if i == 0 { MIX_INPUT } else { MIX_HARMONY };
        let key = routed_note_key(
            target,
            channel_idx,
            u8::from(n),
            mix_group,
            arrangement_slot as u8,
        );
        if count_origin_note_on(origin, output_notes, loop_output_notes, route, key) {
            dispatch_voice(
                target,
                mix_group,
                channel_idx,
                VoiceDispatch::NoteOn {
                    note: u8::from(n),
                    frequency_hz: tuning
                        .as_ref()
                        .and_then(|frame| frame.as_slice().get(i))
                        .map(|pitch| pitch.frequency_hz as f32)
                        .unwrap_or_else(|| standard_frequency(u8::from(n))),
                    velocity: velocity_byte,
                    slide_slot,
                    slide: slide_config.resolve(slide_slot).unwrap_or_default(),
                },
                num_outputs,
                synth_tx,
                output,
                midi_slides,
            );
        }
    }

    // Update shared state — recover from poisoned mutexes rather than panic.
    {
        let mut in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        in_notes.insert(note as u8);
    }
    {
        let mut harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in notes.iter().skip(1) {
            harm_notes.insert(n as u8);
        }
    }
    if engine.last_borrowed_from().is_some() {
        let mut borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in notes.iter().skip(1) {
            borr.insert(n as u8);
        }
    }

    {
        let all_sounding: HashSet<u8> = {
            let in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
            let harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
            in_notes.union(&harm_notes).copied().collect()
        };
        if !all_sounding.is_empty() {
            let key_tonic = Some(engine.key().semitones_from_c());
            let display = chord_display_with_analysis(&all_sounding, key_tonic);
            let mut ch = chord_name.lock().unwrap_or_else(|e| e.into_inner());
            *ch = display;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_note_off(
    channel: Channel,
    note: Note,
    velocity: Velocity,
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    chord_name: &Arc<Mutex<String>>,
    _routing_mode: contrapunk::harmony::RoutingMode,
    synth_tx: &SynthEventSender,
    voice_outputs: &Arc<Mutex<VoiceOutputRoutes>>,
    origin: InputOrigin,
    output_notes: &mut RoutedNoteCounts,
    loop_output_notes: &mut RoutedNoteCounts,
) {
    let notes = engine.harmonize_note_off_owned(note, channel.index());
    let num_outputs = output.connection_count();

    let voice_targets = voice_outputs
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let port_map: Vec<usize> = engine.last_port_map().to_vec();
    let target_for = |i: usize| -> VoiceOutputTarget {
        voice_targets.get(main_voice_route(
            i,
            port_map.get(i).copied().unwrap_or(i) as u8,
        ))
    };

    // Unified per-voice release. Synth NoteOff drops release velocity
    // (SynthEvent::NoteOff has no velocity field); external MIDI
    // preserves it from the input event since some hardware (Yamaha,
    // certain virtual instruments) responds to release velocity.
    let channel_idx: u8 = channel.index();
    let velocity_byte: u8 = u8::from(velocity);
    for (i, &n) in notes.iter().enumerate() {
        let voice_slot = port_map.get(i).copied().unwrap_or(i) as u8;
        let route = main_voice_route(i, voice_slot);
        let target = target_for(i);
        let mix_group = if i == 0 { MIX_INPUT } else { MIX_HARMONY };
        let key = routed_note_key(target, channel_idx, u8::from(n), mix_group, voice_slot);
        if count_origin_note_off(origin, output_notes, loop_output_notes, route, key) {
            dispatch_voice(
                target,
                mix_group,
                channel_idx,
                VoiceDispatch::NoteOff {
                    note: u8::from(n),
                    velocity: velocity_byte,
                    slide_slot: Some(if i == 0 {
                        SlideSlot::new(SlideRole::Input, 0)
                    } else {
                        SlideSlot::new(SlideRole::Harmony, voice_slot)
                    }),
                },
                num_outputs,
                synth_tx,
                output,
                midi_slides,
            );
        }
    }

    {
        let mut in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        in_notes.remove(&(note as u8));
    }
    {
        let mut harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        let mut borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in notes.iter().skip(1) {
            harm_notes.remove(&(n as u8));
            borr.remove(&(n as u8));
        }
    }

    // Recompute chord-name display after the release. Mirrors the
    // identical block in handle_note_on at the all_sounding read site —
    // without this, the previous chord text persists when a note is
    // released, so legato (note-on B before note-off A) shows "A+B"
    // even after A's release leaves only B held. Empties the chord
    // string when nothing is sounding so the UI's `{#if chordName}`
    // checks fall through to the placeholder.
    {
        let all_sounding: HashSet<u8> = {
            let in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
            let harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
            in_notes.union(&harm_notes).copied().collect()
        };
        let mut ch = chord_name.lock().unwrap_or_else(|e| e.into_inner());
        if all_sounding.is_empty() {
            ch.clear();
        } else {
            let key_tonic = Some(engine.key().semitones_from_c());
            *ch = chord_display_with_analysis(&all_sounding, key_tonic);
        }
    }
}

fn standard_frequency(note: u8) -> f32 {
    contrapunk::harmony::tuning::midi_to_frequency(note) as f32
}

/// One per-voice dispatch event consumed by `dispatch_voice`. Carries
/// note + velocity in u8 form (0-127). Channel is passed alongside to
/// the helper since it's typically uniform across a batch of voices
/// (one input event → many voices).
#[derive(Clone, Copy, Debug)]
enum VoiceDispatch {
    /// Send a NoteOn at the given exact synth frequency and velocity.
    NoteOn {
        note: u8,
        frequency_hz: f32,
        velocity: u8,
        slide_slot: SlideSlot,
        slide: SlideSettings,
    },
    /// Send a NoteOff. `velocity` is the release velocity (0 for most
    /// MIDI consumers; some Yamaha hardware uses non-zero release).
    NoteOff {
        note: u8,
        velocity: u8,
        slide_slot: Option<SlideSlot>,
    },
}

/// Single fanout shared by every router-thread NoteOn/NoteOff dispatch
/// site (real-time `handle_note_on` / `handle_note_off`, beat-pattern
/// tick, drain-on-disable, orphan-release-on-retrigger, panic-replay
/// re-attack). Centralises the `target.match { Synth | MidiPort | Off }`
/// skeleton — adding a fourth destination, changing the byte encoding,
/// or instrumenting MIDI traffic now happens in one place.
///
/// Channel and velocity are u8 (0-15 / 0-127). Real-time callers
/// convert from wmidi types at the boundary; pattern / orphan callers
/// pass captured `HeldVoice` fields directly.
fn dispatch_voice(
    target: VoiceOutputTarget,
    mix_group: u8,
    channel: u8,
    event: VoiceDispatch,
    _num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
) {
    // Pin the u7/u4 invariants for callers — every existing site
    // already produces values in range (channel.index() / u8::from
    // on wmidi U7 newtypes), but a future caller passing a raw byte
    // from an untrusted source would silently corrupt the MIDI byte
    // stream without these. Zero release-build cost.
    debug_assert!(channel < 16, "MIDI channel out of range: {}", channel);
    let (n, v) = match event {
        VoiceDispatch::NoteOn { note, velocity, .. } => (note, velocity),
        VoiceDispatch::NoteOff { note, velocity, .. } => (note, velocity),
    };
    debug_assert!(n < 128, "MIDI note out of range: {}", n);
    debug_assert!(v < 128, "MIDI velocity out of range: {}", v);

    match (target, event) {
        (
            VoiceOutputTarget::Synth,
            VoiceDispatch::NoteOn {
                note,
                frequency_hz,
                velocity,
                slide_slot,
                slide,
            },
        ) => {
            let _ = synth_tx.note_on_exact_with_slide(
                note,
                frequency_hz,
                velocity,
                mix_group,
                slide_slot,
                slide,
            );
        }
        (
            VoiceOutputTarget::Synth,
            VoiceDispatch::NoteOff {
                note, slide_slot, ..
            },
        ) => {
            if let Some(slide_slot) = slide_slot {
                let _ = synth_tx.note_off_slot(note, mix_group, slide_slot);
            } else {
                let _ = synth_tx.note_off(note, mix_group);
            }
        }
        (
            VoiceOutputTarget::MidiPort { port },
            VoiceDispatch::NoteOn {
                note,
                frequency_hz,
                velocity,
                slide_slot,
                slide,
            },
        ) => {
            midi_slides.prepare_note_on(
                port,
                channel,
                note,
                frequency_hz,
                slide_slot,
                slide,
                output,
            );
            let msg = [0x90 | (channel & 0x0F), note, velocity];
            if let Err(error) = output.send_to_device_port(port, &msg) {
                eprintln!("[midi] could not send NoteOn to device {port}: {error}");
            }
        }
        (VoiceOutputTarget::MidiPort { port }, VoiceDispatch::NoteOff { note, velocity, .. }) => {
            let msg = [0x80 | (channel & 0x0F), note, velocity];
            if let Err(error) = output.send_to_device_port(port, &msg) {
                eprintln!("[midi] could not send NoteOff to device {port}: {error}");
            }
            midi_slides.note_off(port, channel, note, output);
        }
        (VoiceOutputTarget::Off, _) => {}
    }
}

fn release_routed_key(
    (target, channel, note, mix_group, voice_slot): RoutedNoteKey,
    num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
) {
    let slide_role = match mix_group {
        MIX_CANON => SlideRole::Canon,
        MIX_COUNTERPOINT => SlideRole::Counterpoint,
        MIX_HARMONY => SlideRole::Harmony,
        _ => SlideRole::Input,
    };
    dispatch_voice(
        target,
        mix_group,
        channel,
        VoiceDispatch::NoteOff {
            note,
            velocity: 0,
            slide_slot: Some(SlideSlot::new(slide_role, voice_slot)),
        },
        num_ports,
        synth_tx,
        output,
        midi_slides,
    );
}

fn drain_route_outputs(
    route: VoiceRouteId,
    notes: &mut RoutedNoteCounts,
    loop_notes: &mut RoutedNoteCounts,
    num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
) {
    loop_notes.remove_route(route);
    for key in notes.remove_route(route) {
        release_routed_key(key, num_ports, synth_tx, output, midi_slides);
    }
}

fn drain_routed_outputs(
    notes: &mut RoutedNoteCounts,
    num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
) {
    for key in notes.drain() {
        release_routed_key(key, num_ports, synth_tx, output, midi_slides);
    }
}

fn send_all_notes_off(num_ports: usize, synth_tx: &SynthEventSender, output: &mut OutputRouter) {
    let _ = synth_tx.send(SynthEvent::AllNotesOff);
    for channel in 0u8..16 {
        let message = [0xb0 | channel, 123, 0];
        for port in 0..num_ports {
            if let Err(error) = output.send_to_port(port, &message) {
                eprintln!("[midi] could not send All Notes Off to connection {port}: {error}");
            }
        }
    }
}

fn cleanup_loop_outputs(
    all_notes: &mut RoutedNoteCounts,
    loop_notes: &mut RoutedNoteCounts,
    sustain_owners: &mut SustainOwners,
    num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
) {
    for key in all_notes.remove_owned(loop_notes) {
        release_routed_key(key, num_ports, synth_tx, output, midi_slides);
    }

    let mut loop_sustain: Vec<_> = sustain_owners
        .iter()
        .copied()
        .filter(|(origin, _, _)| *origin == InputOrigin::Loop)
        .collect();
    loop_sustain.sort_unstable();
    for owner @ (_, target, channel) in loop_sustain {
        sustain_owners.remove(&owner);
        if !sustain_owners
            .iter()
            .any(|(_, other_target, other_channel)| {
                *other_target == target && *other_channel == channel
            })
        {
            dispatch_sustain(
                target,
                channel,
                false,
                num_ports,
                synth_tx,
                output,
                midi_slides,
            );
        }
    }
}

fn clear_all_sustain(
    owners: &mut SustainOwners,
    num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
) {
    let mut destinations: Vec<_> = owners
        .iter()
        .map(|(_, target, channel)| (*target, *channel))
        .collect();
    destinations.sort_unstable();
    destinations.dedup();
    owners.clear();
    for (target, channel) in destinations {
        dispatch_sustain(
            target,
            channel,
            false,
            num_ports,
            synth_tx,
            output,
            midi_slides,
        );
    }
}

fn dispatch_sustain(
    target: VoiceOutputTarget,
    channel: u8,
    on: bool,
    _num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
) {
    match target {
        VoiceOutputTarget::Synth => {
            let _ = synth_tx.send(SynthEvent::sustain_pedal(on));
        }
        VoiceOutputTarget::MidiPort { port } => {
            let message = [0xb0 | (channel & 0x0f), 64, if on { 127 } else { 0 }];
            if let Err(error) = output.send_to_device_port(port, &message) {
                eprintln!("[midi] could not send sustain to device {port}: {error}");
            }
            midi_slides.set_sustain(port, channel, on, output);
        }
        VoiceOutputTarget::Off => {}
    }
}

fn set_sustain_ownership(
    origin: InputOrigin,
    channel: u8,
    on: bool,
    owners: &mut SustainOwners,
    voice_outputs: &Arc<Mutex<VoiceOutputRoutes>>,
    num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
) {
    let routes = voice_outputs
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut targets = vec![(VoiceOutputTarget::Synth, 0)];
    targets.extend(routes.assignments().filter_map(|(route, target)| {
        matches!(routes.get(route), VoiceOutputTarget::MidiPort { .. }).then_some((target, channel))
    }));
    targets.sort_unstable();
    targets.dedup();
    for (target, routed_channel) in targets {
        let owner = (origin, target, routed_channel);
        if on {
            let first = !owners.iter().any(|(_, owned_target, owned_channel)| {
                *owned_target == target && *owned_channel == routed_channel
            });
            if owners.insert(owner) && first {
                dispatch_sustain(
                    target,
                    routed_channel,
                    true,
                    num_ports,
                    synth_tx,
                    output,
                    midi_slides,
                );
            }
        } else if owners.remove(&owner)
            && !owners.iter().any(|(_, owned_target, owned_channel)| {
                *owned_target == target && *owned_channel == routed_channel
            })
        {
            dispatch_sustain(
                target,
                routed_channel,
                false,
                num_ports,
                synth_tx,
                output,
                midi_slides,
            );
        }
    }
}

fn midi_bytes_to_loop_event(bytes: &[u8], origin: InputOrigin) -> Option<OriginMidiEvent> {
    if bytes.len() < 3 || bytes[1] >= 128 || bytes[2] >= 128 {
        return None;
    }
    let channel = bytes[0] & 0x0f;
    let event = match bytes[0] & 0xf0 {
        0x90 if bytes[2] > 0 => LoopMidiEvent::NoteOn {
            note: bytes[1],
            velocity: bytes[2],
            channel,
        },
        0x80 | 0x90 => LoopMidiEvent::NoteOff {
            note: bytes[1],
            velocity: bytes[2],
            channel,
        },
        0xb0 if bytes[1] == 64 => LoopMidiEvent::Cc64 {
            value: bytes[2],
            channel,
        },
        _ => return None,
    };
    Some(OriginMidiEvent {
        origin,
        event,
        scheduled_beat_us: None,
    })
}

fn loop_event_bytes(event: LoopMidiEvent) -> [u8; 3] {
    match event {
        LoopMidiEvent::NoteOn {
            note,
            velocity,
            channel,
        } => [0x90 | channel, note, velocity],
        LoopMidiEvent::NoteOff {
            note,
            velocity,
            channel,
        } => [0x80 | channel, note, velocity],
        LoopMidiEvent::Cc64 { value, channel } => [0xb0 | channel, 64, value],
    }
}

/// Translate raw MIDI bytes into a Companion `InputEvent` for the
/// `on_input` pipeline. Returns `None` for messages that aren't one
/// of the three supported event types (NoteOn / NoteOff / Cc) — those
/// fall through to the legacy router path untouched.
///
/// Pure: takes a byte slice, returns Option. No I/O, no locks.
fn midi_bytes_to_input_event(bytes: &[u8]) -> Option<crate::companion::InputEvent> {
    use crate::companion::InputEvent;
    if bytes.len() < 3 {
        return None;
    }
    let status = bytes[0] & 0xF0;
    let channel = bytes[0] & 0x0F;
    match status {
        // NoteOn with velocity 0 is the wmidi-equivalent NoteOff —
        // honor that convention here too so Lanes see a consistent
        // input shape across controllers.
        0x90 if bytes[2] != 0 => Some(InputEvent::NoteOn {
            note: bytes[1],
            velocity: bytes[2],
            channel,
        }),
        0x80 | 0x90 => Some(InputEvent::NoteOff {
            note: bytes[1],
            channel,
        }),
        0xB0 => Some(InputEvent::Cc {
            number: bytes[1],
            value: bytes[2],
            channel,
        }),
        _ => None,
    }
}

/// Translate `Companion::tick`'s `DispatchOp` outputs into the
/// existing `dispatch_voice` calls so Lanes
/// can drive notes through the same routing fabric harmony voices
/// use. Pure dispatch — no decisions, no state mutation beyond the
/// existing per-call helpers.
///
/// `DispatchOp::AllNotesOff { ports }` ignores its `ports` list for
/// now and broadcasts to every connected output (matches the existing
/// CC 123 panic-drain behavior in the router). Per-port targeting
/// is deferred until the audio-graph milestone introduces an
/// `InstrumentId` model.
fn dispatch_companion_ops(
    tagged: &[(&'static str, u8, crate::companion::DispatchOp)],
    num_ports: usize,
    synth_tx: &SynthEventSender,
    output: &mut OutputRouter,
    midi_slides: &mut MidiSlideRuntime,
    voice_outputs: &Arc<Mutex<VoiceOutputRoutes>>,
    _harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    canon_notes: &Arc<Mutex<NoteCounts>>,
    counterpoint_notes: &Arc<Mutex<NoteCounts>>,
    output_notes: &mut RoutedNoteCounts,
    loop_output_notes: &mut RoutedNoteCounts,
    origin: InputOrigin,
    slide_config: &Arc<Mutex<SlideConfig>>,
) {
    use crate::companion::DispatchOp;
    let slide_config = *slide_config
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let voice_outputs = voice_outputs
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    for (lane, voice_slot, op) in tagged {
        // Routing identity stays separate from the synth mix grouping: pattern
        // lanes keep their own destinations even though they share a mixer role.
        let (lane_notes, mix_group) = match *lane {
            "canon" | "pattern_low" => (Some(canon_notes), MIX_CANON),
            "counterpoint" | "pattern_counter" => (Some(counterpoint_notes), MIX_COUNTERPOINT),
            _ => (None, MIX_HARMONY),
        };
        let route = companion_voice_route(lane, *voice_slot);
        let routed_target = voice_outputs.get(route);
        match op {
            DispatchOp::NoteOn {
                note,
                velocity,
                channel,
                ..
            } => {
                let first_owner = count_origin_note_on(
                    origin,
                    output_notes,
                    loop_output_notes,
                    route,
                    routed_note_key(routed_target, *channel, *note, mix_group, *voice_slot),
                );
                if first_owner {
                    let slide_role = if mix_group == MIX_CANON {
                        SlideRole::Canon
                    } else {
                        SlideRole::Counterpoint
                    };
                    let slide_slot = SlideSlot::new(slide_role, *voice_slot);
                    dispatch_voice(
                        routed_target,
                        mix_group,
                        *channel,
                        VoiceDispatch::NoteOn {
                            note: *note,
                            frequency_hz: standard_frequency(*note),
                            velocity: *velocity,
                            slide_slot,
                            slide: slide_config.resolve(slide_slot).unwrap_or_default(),
                        },
                        num_ports,
                        synth_tx,
                        output,
                        midi_slides,
                    );
                }
                if let Some(notes) = lane_notes {
                    let mut notes = notes.lock().unwrap_or_else(|e| e.into_inner());
                    count_note_on(&mut notes, *note);
                }
            }
            DispatchOp::NoteOff { note, channel, .. } => {
                let last_owner = count_origin_note_off(
                    origin,
                    output_notes,
                    loop_output_notes,
                    route,
                    routed_note_key(routed_target, *channel, *note, mix_group, *voice_slot),
                );
                if last_owner {
                    dispatch_voice(
                        routed_target,
                        mix_group,
                        *channel,
                        VoiceDispatch::NoteOff {
                            note: *note,
                            velocity: 0,
                            slide_slot: Some(SlideSlot::new(
                                if mix_group == MIX_CANON {
                                    SlideRole::Canon
                                } else {
                                    SlideRole::Counterpoint
                                },
                                *voice_slot,
                            )),
                        },
                        num_ports,
                        synth_tx,
                        output,
                        midi_slides,
                    );
                }
                if let Some(notes) = lane_notes {
                    let mut notes = notes.lock().unwrap_or_else(|e| e.into_inner());
                    count_note_off(&mut notes, *note);
                }
            }
            DispatchOp::AllNotesOff { .. } if origin == InputOrigin::Live => {
                output_notes.clear();
                loop_output_notes.clear();
                midi_slides.clear(output);
                if let Some(notes) = lane_notes {
                    notes.lock().unwrap_or_else(|e| e.into_inner()).clear();
                }
                send_all_notes_off(num_ports, synth_tx, output);
            }
            DispatchOp::AllNotesOff { .. } => {}
        }
    }
}

/// Drain every tracked note source and return the pitch union so the
/// caller can dispatch NoteOff for each. Handles the
/// CC 123 (All Notes Off) panic path in run_tauri_router.
///
/// Acquires locks in a fixed order (input → harmony → borrowed →
/// canon → counterpoint) to avoid deadlock with the rest of the router which
/// also reads them in similar order. Recovers from poisoned mutexes
/// rather than panicking — matches the convention in the router's
/// emit loop.
fn drain_all_tracked_notes(
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    canon_notes: &Arc<Mutex<NoteCounts>>,
    counterpoint_notes: &Arc<Mutex<NoteCounts>>,
) -> Vec<u8> {
    let union: HashSet<u8> = {
        let in_n = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        let harm = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        let borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        let canon = canon_notes.lock().unwrap_or_else(|e| e.into_inner());
        let counterpoint = counterpoint_notes.lock().unwrap_or_else(|e| e.into_inner());
        in_n.iter()
            .chain(harm.iter())
            .chain(borr.iter())
            .copied()
            .chain(canon.keys().copied())
            .chain(counterpoint.keys().copied())
            .collect()
    };
    // Now clear each set (separate scope so the prior read locks
    // are dropped before we acquire write locks).
    {
        let mut s = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        s.clear();
    }
    {
        let mut s = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        s.clear();
    }
    {
        let mut s = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        s.clear();
    }
    {
        let mut s = canon_notes.lock().unwrap_or_else(|e| e.into_inner());
        s.clear();
    }
    {
        let mut s = counterpoint_notes.lock().unwrap_or_else(|e| e.into_inner());
        s.clear();
    }
    let mut union: Vec<_> = union.into_iter().collect();
    union.sort_unstable();
    union
}

/// Detect the issue #14 silent-no-output condition: user selected one
/// or more external MIDI ports for routing, but no voice in the
/// per-voice routing table actually points at an external port. The
/// engine's per-voice routing (`VoiceOutputTarget::default() = Synth`)
/// will silently swallow every harmony into the internal synth, which
/// looks like "MIDI out broken" to the user.
///
/// Returns `Some(message)` describing the problem when the bad state
/// is detected, or `None` when configuration is consistent.
///
/// Pure: takes slices, returns Option<String>. No I/O, no locks.
fn detect_no_external_output_warning(
    voice_outputs: &VoiceOutputRoutes,
    output_indices: &[usize],
) -> Option<String> {
    if output_indices.is_empty() {
        // No external ports selected at all — synth-only is what the
        // user picked. Don't warn.
        return None;
    }
    if voice_outputs.has_external_target() {
        return None;
    }
    Some(format!(
        "Routing started with {} external MIDI port(s) selected, but no voice is routed to an external port. \
         All voices currently route to the internal synth; external instruments will receive nothing. \
         Open Voice Routing in the UI to send harmonies to your external port(s).",
        output_indices.len()
    ))
}

/// Build the payload sent on the "note-update" Tauri event.
///
/// Pure function: takes references to the note sets/counters + the
/// strings already extracted from the engine, returns the payload.
/// The router holds the locks; this function never blocks on I/O.
///
/// Sorts each note vector for stable UI rendering — without sorting,
/// HashSet iteration order would cause the piano-roll display to
/// shuffle pips between frames.
#[allow(clippy::too_many_arguments)]
fn build_note_update_payload(
    input_notes: &HashSet<u8>,
    harmony_notes: &HashSet<u8>,
    borrowed_notes: &HashSet<u8>,
    chord_name: String,
    last_borrowed_from: String,
    current_key: String,
    canon_notes: &NoteCounts,
    counterpoint_notes: &NoteCounts,
    phrase: contrapunk_companion::PhraseSnapshot,
) -> NoteUpdatePayload {
    let mut in_vec: Vec<u8> = input_notes.iter().copied().collect();
    let mut harm_vec: Vec<u8> = harmony_notes.iter().copied().collect();
    let mut borr_vec: Vec<u8> = borrowed_notes.iter().copied().collect();
    let mut canon_vec: Vec<u8> = canon_notes.keys().copied().collect();
    let mut cp_vec: Vec<u8> = counterpoint_notes.keys().copied().collect();
    in_vec.sort_unstable();
    harm_vec.sort_unstable();
    borr_vec.sort_unstable();
    canon_vec.sort_unstable();
    cp_vec.sort_unstable();
    NoteUpdatePayload {
        input_notes: in_vec,
        harmony_notes: harm_vec,
        borrowed_notes: borr_vec,
        chord_name,
        last_borrowed_from,
        current_key,
        canon_notes: canon_vec,
        counterpoint_notes: cp_vec,
        phrase,
    }
}

/// Build the payload sent on the "guitar-signal" Tauri event.
///
/// Pure function: converts a `GuitarSignalInfo` from the audio
/// pipeline into the UI-facing payload, applying the standard
/// frequency-to-note mapping with two thresholds (frequency > 20 Hz
/// AND clarity > 0.3) to avoid surfacing noise as fake note
/// detections in the readout strip.
fn build_guitar_signal_payload(sig: crate::guitar_bridge::GuitarSignalInfo) -> GuitarSignalPayload {
    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
    ];
    let (note_name, midi_note) = if let Some(freq) = sig.frequency {
        if freq > 20.0 && sig.clarity > 0.3 {
            let midi = (12.0 * (freq / 440.0).log2() + 69.0).round() as i32;
            let midi_u8 = midi.clamp(0, 127) as u8;
            let name_idx = (midi_u8 % 12) as usize;
            let octave = (midi_u8 as i32 / 12) - 1;
            (format!("{}{}", NOTE_NAMES[name_idx], octave), midi_u8)
        } else {
            (String::new(), 0)
        }
    } else {
        (String::new(), 0)
    };
    GuitarSignalPayload {
        rms: sig.rms,
        frequency: sig.frequency,
        clarity: sig.clarity,
        note_state: sig.note_state,
        note_name,
        midi_note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guitar_bridge::GuitarSignalInfo;
    use std::collections::HashSet;

    /// Note vectors must come out sorted ascending so the UI piano-roll
    /// renders pips in a stable order across frames. Without sorting,
    /// HashSet iteration order would shuffle the display.
    #[test]
    fn test_build_note_update_payload_sorts_notes() {
        let input: HashSet<u8> = [67u8, 60, 64].iter().copied().collect();
        let harmony: HashSet<u8> = [55u8, 71, 60].iter().copied().collect();
        let borrowed: HashSet<u8> = [70u8].iter().copied().collect();
        let canon: NoteCounts = [(71u8, 2), (60, 1)].into_iter().collect();
        let cp: NoteCounts = [(55u8, 1)].into_iter().collect();
        let payload = build_note_update_payload(
            &input,
            &harmony,
            &borrowed,
            "Cmaj7".into(),
            "Aeolian".into(),
            "C".into(),
            &canon,
            &cp,
            contrapunk_companion::PhraseSnapshot::idle(
                contrapunk_companion::DEFAULT_PHRASE_GAP_BEATS,
            ),
        );
        assert_eq!(payload.input_notes, vec![60, 64, 67]);
        assert_eq!(payload.harmony_notes, vec![55, 60, 71]);
        assert_eq!(payload.borrowed_notes, vec![70]);
        assert_eq!(payload.canon_notes, vec![60, 71]);
        assert_eq!(payload.counterpoint_notes, vec![55]);
        assert_eq!(payload.chord_name, "Cmaj7");
        assert_eq!(payload.last_borrowed_from, "Aeolian");
        assert_eq!(payload.current_key, "C");
    }

    #[test]
    fn test_build_note_update_payload_empty_state() {
        let empty: HashSet<u8> = HashSet::new();
        let empty_counts: NoteCounts = HashMap::new();
        let payload = build_note_update_payload(
            &empty,
            &empty,
            &empty,
            String::new(),
            String::new(),
            "C".into(),
            &empty_counts,
            &empty_counts,
            contrapunk_companion::PhraseSnapshot::idle(
                contrapunk_companion::DEFAULT_PHRASE_GAP_BEATS,
            ),
        );
        assert!(payload.input_notes.is_empty());
        assert!(payload.harmony_notes.is_empty());
        assert!(payload.borrowed_notes.is_empty());
        assert!(payload.canon_notes.is_empty());
        assert!(payload.counterpoint_notes.is_empty());
        assert!(payload.chord_name.is_empty());
        assert_eq!(payload.current_key, "C");
    }

    #[test]
    fn note_counts_preserve_overlapping_lane_voices() {
        let mut notes = NoteCounts::new();
        count_note_on(&mut notes, 64);
        count_note_on(&mut notes, 64);
        count_note_off(&mut notes, 64);
        assert_eq!(notes.get(&64), Some(&1));
        count_note_off(&mut notes, 64);
        assert!(!notes.contains_key(&64));
    }

    #[test]
    fn routed_note_counts_hold_same_pitch_until_last_owner() {
        let mut notes = RoutedNoteCounts::new();
        let first = routed_note_key(VoiceOutputTarget::Synth, 5, 64, MIX_CANON, 0);
        let second = routed_note_key(VoiceOutputTarget::Synth, 6, 64, MIX_CANON, 0);
        assert_eq!(
            first, second,
            "synth ownership collapses channels within a role"
        );
        assert_ne!(
            first,
            routed_note_key(VoiceOutputTarget::Synth, 5, 64, MIX_COUNTERPOINT, 0),
            "different mixer roles keep independent synth voices"
        );
        assert_ne!(
            first,
            routed_note_key(VoiceOutputTarget::Synth, 5, 64, MIX_CANON, 1),
            "generated slots keep equal pitches independently owned"
        );
        let route = VoiceRouteId::Canon { voice: 0 };
        assert!(count_routed_note_on(&mut notes, route, first));
        assert!(!count_routed_note_on(&mut notes, route, second));
        assert!(!count_routed_note_off(&mut notes, route, first));
        assert!(count_routed_note_off(&mut notes, route, second));
    }

    #[test]
    fn stale_harmony_release_never_steals_equal_input_pitch() {
        let mut notes = RoutedNoteCounts::new();
        let input_key = routed_note_key(VoiceOutputTarget::Synth, 0, 60, MIX_INPUT, 0);
        let harmony_key = routed_note_key(VoiceOutputTarget::Synth, 0, 60, MIX_HARMONY, 1);
        notes.note_on(VoiceRouteId::Input, input_key);
        notes.note_on(VoiceRouteId::Harmony { slot: 1 }, harmony_key);

        assert_eq!(notes.remove_harmony_note(60), [harmony_key]);
        assert!(notes.owns(VoiceRouteId::Input, input_key));
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn draining_one_route_preserves_shared_physical_owner() {
        let mut notes = RoutedNoteCounts::new();
        let key = routed_note_key(VoiceOutputTarget::MidiPort { port: 4 }, 2, 64, MIX_INPUT, 0);
        let first = VoiceRouteId::Canon { voice: 0 };
        let second = VoiceRouteId::Canon { voice: 1 };
        assert!(notes.note_on(first, key));
        assert!(!notes.note_on(second, key));

        assert!(notes.remove_route(first).is_empty());
        assert_eq!(notes.len(), 1);
        assert!(notes.note_off(second, key));
        assert!(notes.is_empty());
    }

    /// A clean signal at A4 (440 Hz) above the clarity threshold must
    /// produce the canonical name + MIDI number.
    #[test]
    fn test_build_guitar_signal_payload_a4() {
        let sig = GuitarSignalInfo {
            rms: 0.1,
            frequency: Some(440.0),
            clarity: 0.8,
            note_state: 2,
        };
        let payload = build_guitar_signal_payload(sig);
        assert_eq!(payload.note_name, "A4");
        assert_eq!(payload.midi_note, 69);
        assert_eq!(payload.note_state, 2);
    }

    /// Frequency below 20 Hz must NOT produce a note — that's
    /// sub-audible noise. The clarity threshold should also gate.
    #[test]
    fn test_build_guitar_signal_payload_rejects_subaudible_and_low_clarity() {
        let subaudible = GuitarSignalInfo {
            rms: 0.05,
            frequency: Some(10.0),
            clarity: 0.9,
            note_state: 0,
        };
        let p1 = build_guitar_signal_payload(subaudible);
        assert_eq!(p1.note_name, "");
        assert_eq!(p1.midi_note, 0);

        let noisy = GuitarSignalInfo {
            rms: 0.05,
            frequency: Some(440.0),
            clarity: 0.2,
            note_state: 0,
        };
        let p2 = build_guitar_signal_payload(noisy);
        assert_eq!(p2.note_name, "");
        assert_eq!(p2.midi_note, 0);
    }

    /// CC 123 panic must drain ALL three tracked sets — input, harmony,
    /// and borrowed. The function returns the union so callers can fan
    /// NoteOff to every downstream port.
    #[test]
    fn test_drain_all_tracked_notes_collects_union_and_clears() {
        let input = Arc::new(Mutex::new(
            [60u8, 64].iter().copied().collect::<HashSet<u8>>(),
        ));
        let harmony = Arc::new(Mutex::new(
            [67u8, 71].iter().copied().collect::<HashSet<u8>>(),
        ));
        let borrowed = Arc::new(Mutex::new([70u8].iter().copied().collect::<HashSet<u8>>()));
        let canon = Arc::new(Mutex::new([(72u8, 2)].into_iter().collect::<NoteCounts>()));
        let counterpoint = Arc::new(Mutex::new([(53u8, 1)].into_iter().collect::<NoteCounts>()));
        let drained = drain_all_tracked_notes(&input, &harmony, &borrowed, &canon, &counterpoint);
        assert_eq!(drained, vec![53, 60, 64, 67, 70, 71, 72]);
        assert!(input.lock().unwrap().is_empty());
        assert!(harmony.lock().unwrap().is_empty());
        assert!(borrowed.lock().unwrap().is_empty());
        assert!(canon.lock().unwrap().is_empty());
        assert!(counterpoint.lock().unwrap().is_empty());
    }

    /// Empty sets must produce an empty union without panicking.
    #[test]
    fn test_drain_all_tracked_notes_empty_returns_empty() {
        let input = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let harmony = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let borrowed = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let canon = Arc::new(Mutex::new(NoteCounts::new()));
        let counterpoint = Arc::new(Mutex::new(NoteCounts::new()));
        let drained = drain_all_tracked_notes(&input, &harmony, &borrowed, &canon, &counterpoint);
        assert!(drained.is_empty());
    }

    /// Overlapping notes across sets must dedupe in the union (HashSet
    /// semantics) so the caller doesn't fire duplicate NoteOffs.
    #[test]
    fn test_drain_all_tracked_notes_dedups_overlaps() {
        let input = Arc::new(Mutex::new([60u8].iter().copied().collect::<HashSet<u8>>()));
        let harmony = Arc::new(Mutex::new(
            [60u8, 64].iter().copied().collect::<HashSet<u8>>(),
        ));
        let borrowed = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let canon = Arc::new(Mutex::new([(60u8, 2)].into_iter().collect::<NoteCounts>()));
        let counterpoint = Arc::new(Mutex::new(NoteCounts::new()));
        let drained = drain_all_tracked_notes(&input, &harmony, &borrowed, &canon, &counterpoint);
        assert_eq!(drained, vec![60, 64]);
    }

    /// Regression: `drain_all_tracked_notes` must recover from a
    /// poisoned mutex via the established `unwrap_or_else(|e|
    /// e.into_inner())` pattern (and equivalent), not panic. Without
    /// this, a panic in any thread that holds these locks (e.g. an
    /// inner unwrap fault during reharm) cascades into a dead router.
    /// First poison-recovery test in the project — proves the
    /// convention on one site is enough to prove it on all.
    #[test]
    fn test_drain_all_tracked_notes_survives_poisoned_lock() {
        use std::thread;
        let input = Arc::new(Mutex::new([60u8].iter().copied().collect::<HashSet<u8>>()));
        let harmony = Arc::new(Mutex::new(
            [60u8, 64].iter().copied().collect::<HashSet<u8>>(),
        ));
        let borrowed = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let canon = Arc::new(Mutex::new(NoteCounts::new()));
        let counterpoint = Arc::new(Mutex::new(NoteCounts::new()));

        // Poison the middle lock from a spawned thread by holding it
        // while panicking. .join() catches the panic so the test
        // process survives.
        let poisoner = {
            let h = Arc::clone(&harmony);
            thread::spawn(move || {
                let _guard = h.lock().unwrap();
                panic!("intentional poison for test");
            })
        };
        let _ = poisoner.join();
        assert!(
            harmony.is_poisoned(),
            "poisoner thread should have flagged the mutex"
        );

        // drain_all_tracked_notes must still see the wrapped data
        // (60, 64 from the harmony set) instead of panicking.
        let drained = drain_all_tracked_notes(&input, &harmony, &borrowed, &canon, &counterpoint);
        assert_eq!(
            drained,
            vec![60, 64],
            "drain must recover via .unwrap_or_else(|e| e.into_inner()) on poisoned mutex"
        );
    }

    /// MIDI NoteOn (status 0x9X) with non-zero velocity decodes to
    /// the InputEvent::NoteOn variant carrying note + velocity +
    /// channel. The channel is the low nibble of the status byte.
    #[test]
    fn test_midi_bytes_to_input_event_note_on() {
        use crate::companion::InputEvent;
        let ev = midi_bytes_to_input_event(&[0x91, 60, 100]).unwrap();
        match ev {
            InputEvent::NoteOn {
                note,
                velocity,
                channel,
            } => {
                assert_eq!(note, 60);
                assert_eq!(velocity, 100);
                assert_eq!(channel, 1);
            }
            _ => panic!("expected NoteOn, got {:?}", ev),
        }
    }

    /// MIDI convention: NoteOn with velocity 0 is a NoteOff. Verify
    /// that decodes correctly so Lanes see a normalized input shape
    /// regardless of which form the controller sent.
    #[test]
    fn test_midi_bytes_to_input_event_note_on_zero_velocity_is_off() {
        use crate::companion::InputEvent;
        let ev = midi_bytes_to_input_event(&[0x90, 60, 0]).unwrap();
        assert!(matches!(
            ev,
            InputEvent::NoteOff {
                note: 60,
                channel: 0
            }
        ));
    }

    /// Explicit NoteOff (status 0x8X) decodes regardless of velocity.
    #[test]
    fn test_midi_bytes_to_input_event_note_off() {
        use crate::companion::InputEvent;
        let ev = midi_bytes_to_input_event(&[0x82, 64, 50]).unwrap();
        assert!(matches!(
            ev,
            InputEvent::NoteOff {
                note: 64,
                channel: 2
            }
        ));
    }

    /// Control Change (status 0xBX) decodes to InputEvent::Cc.
    #[test]
    fn test_midi_bytes_to_input_event_cc() {
        use crate::companion::InputEvent;
        let ev = midi_bytes_to_input_event(&[0xB3, 7, 100]).unwrap();
        match ev {
            InputEvent::Cc {
                number,
                value,
                channel,
            } => {
                assert_eq!(number, 7);
                assert_eq!(value, 100);
                assert_eq!(channel, 3);
            }
            _ => panic!("expected Cc, got {:?}", ev),
        }
    }

    /// Non-note status bytes are not Companion input events. The router
    /// handles pitch bend separately so it can compose with tuning and Slide.
    #[test]
    fn test_midi_bytes_to_input_event_passthrough() {
        assert!(midi_bytes_to_input_event(&[0xE0, 0, 64]).is_none()); // pitch bend
        assert!(midi_bytes_to_input_event(&[0xD0, 100, 0]).is_none()); // channel pressure
        assert!(midi_bytes_to_input_event(&[]).is_none());
        assert!(midi_bytes_to_input_event(&[0x90]).is_none()); // too short
    }

    #[test]
    fn midi_slide_uses_48_semitone_bend_range_and_exact_endpoints() {
        assert_eq!(midi_bend_value(0.0, 48.0), 8_192);
        assert_eq!(midi_bend_value(12.0, 48.0), 10_240);
        assert_eq!(midi_bend_value(-48.0, 48.0), 0);
        assert_eq!(midi_bend_value(48.0, 48.0), 16_383);
        assert_eq!(midi_bend_value(96.0, 48.0), 16_383);
        assert_eq!(midi_bend_value(1.0, 2.0), 12_288);
    }

    #[test]
    fn midi_slide_preserves_tuned_frequency_and_configures_receiver_range() {
        assert!((midi_frequency_semitones(69, 880.0) - 12.0).abs() < 1.0e-4);
        assert_eq!(midi_input_bend_semitones(8_192), 0.0);
        assert_eq!(midi_input_bend_semitones(12_288), 1.0);
        assert_eq!(
            midi_bend_range_messages(3, 48),
            [
                [0xb3, 101, 0],
                [0xb3, 100, 0],
                [0xb3, 6, 48],
                [0xb3, 38, 0],
                [0xb3, 101, 127],
                [0xb3, 100, 127],
            ]
        );
    }

    #[test]
    fn midi_slide_composes_tuning_detune_and_input_bend_when_slide_is_off() {
        let mut output = OutputRouter::new(&[]).unwrap();
        let mut runtime = MidiSlideRuntime::new(25, Arc::new(SlideTelemetry::new()));
        runtime.set_input_bend(0, 0, 0.5, &mut output);
        let target_hz = standard_frequency(69) * 2.0_f32.powf(0.25 / 12.0);

        runtime.prepare_note_on(
            0,
            0,
            69,
            target_hz,
            SlideSlot::new(SlideRole::Input, 0),
            SlideSettings::default(),
            &mut output,
        );

        assert_eq!(runtime.last_bends.get(&(0, 0)), Some(&12_288));
        assert!(runtime.configured_channels.is_empty());
    }

    #[test]
    fn midi_slide_note_off_restores_the_remaining_channel_owner() {
        let mut output = OutputRouter::new(&[]).unwrap();
        let mut runtime = MidiSlideRuntime::new(0, Arc::new(SlideTelemetry::new()));
        let settings = SlideSettings::default();
        let slot = SlideSlot::new(SlideRole::Harmony, 0);
        runtime.prepare_note_on(
            0,
            0,
            60,
            standard_frequency(60),
            slot,
            settings,
            &mut output,
        );
        runtime.prepare_note_on(
            0,
            0,
            64,
            standard_frequency(64) * 2.0_f32.powf(1.0 / 12.0),
            slot,
            settings,
            &mut output,
        );
        assert_eq!(runtime.last_bends.get(&(0, 0)), Some(&12_288));

        runtime.note_off(0, 0, 64, &mut output);

        assert_eq!(runtime.voices.len(), 1);
        assert_eq!(runtime.last_bends.get(&(0, 0)), Some(&8_192));
    }

    #[test]
    fn midi_slide_tick_never_overrides_a_newer_stationary_channel_owner() {
        let mut output = OutputRouter::new(&[]).unwrap();
        let telemetry = Arc::new(SlideTelemetry::new());
        let mut runtime = MidiSlideRuntime::new(0, Arc::clone(&telemetry));
        let moving_slot = SlideSlot::new(SlideRole::Harmony, 0);
        runtime.prepare_note_on(
            0,
            0,
            60,
            standard_frequency(60),
            moving_slot,
            SlideSettings::default(),
            &mut output,
        );
        runtime.note_off(0, 0, 60, &mut output);
        runtime.prepare_note_on(
            0,
            0,
            72,
            standard_frequency(72),
            moving_slot,
            SlideSettings {
                travel: SlideTravel::Time {
                    milliseconds: 1_000.0,
                },
                trigger: contrapunk::slide::SlideTrigger::Always,
                curve: Default::default(),
            },
            &mut output,
        );
        let moving = telemetry.snapshot();
        assert_eq!(moving.len(), 1);
        assert_eq!(moving[0].slot, moving_slot);
        assert!(moving[0].voice_id >= 1 << 63);

        runtime.prepare_note_on(
            0,
            0,
            67,
            standard_frequency(67),
            SlideSlot::new(SlideRole::Harmony, 1),
            SlideSettings::default(),
            &mut output,
        );
        assert_eq!(runtime.last_bends.get(&(0, 0)), Some(&8_192));

        runtime.last_tick = Instant::now() - MIDI_SLIDE_UPDATE_INTERVAL;
        runtime.tick(&mut output);

        assert_eq!(runtime.last_bends.get(&(0, 0)), Some(&8_192));
    }

    #[test]
    fn midi_slide_keeps_sustained_voice_until_pedal_up() {
        let mut output = OutputRouter::new(&[]).unwrap();
        let mut runtime = MidiSlideRuntime::new(0, Arc::new(SlideTelemetry::new()));
        runtime.prepare_note_on(
            0,
            2,
            60,
            standard_frequency(60),
            SlideSlot::new(SlideRole::Input, 0),
            SlideSettings::default(),
            &mut output,
        );
        runtime.set_sustain(0, 2, true, &mut output);
        runtime.note_off(0, 2, 60, &mut output);

        assert_eq!(runtime.voices.len(), 1);
        assert!(!runtime.voices[0].key_down);

        runtime.set_sustain(0, 2, false, &mut output);
        assert!(runtime.voices.is_empty());
        assert_eq!(runtime.last_bends.get(&(0, 2)), Some(&8_192));
    }

    #[test]
    fn routed_cleanup_releases_canon_on_its_original_mpe_channel() {
        let mut output = OutputRouter::new(&[]).unwrap();
        let mut runtime = MidiSlideRuntime::new(0, Arc::new(SlideTelemetry::new()));
        runtime.prepare_note_on(
            0,
            6,
            60,
            standard_frequency(60),
            SlideSlot::new(SlideRole::Canon, 0),
            SlideSettings::default(),
            &mut output,
        );
        let mut notes = RoutedNoteCounts::new();
        notes.note_on(
            VoiceRouteId::Canon { voice: 0 },
            routed_note_key(VoiceOutputTarget::MidiPort { port: 0 }, 6, 60, MIX_CANON, 0),
        );
        let (synth_tx, _synth_rx) = contrapunk::elixir::synth_event_channel();

        drain_routed_outputs(&mut notes, 0, &synth_tx, &mut output, &mut runtime);

        assert!(notes.is_empty());
        assert!(runtime.voices.is_empty());
    }

    #[test]
    fn cc_panic_rebroadcast_guard_breaks_immediate_iac_feedback() {
        let first = Instant::now();
        assert!(should_rebroadcast_cc_panic(None, first));
        assert!(!should_rebroadcast_cc_panic(
            Some(first),
            first + Duration::from_millis(249)
        ));
        assert!(should_rebroadcast_cc_panic(
            Some(first),
            first + Duration::from_millis(250)
        ));
    }

    #[test]
    fn passthrough_bytes_follow_the_input_route_instead_of_the_first_port() {
        let routes = Arc::new(Mutex::new(VoiceOutputRoutes::default()));
        let mut output = OutputRouter::recording(&[4, 9]);
        send_input_passthrough(&[0xf8], &routes, &mut output);
        assert!(output.take_trace().is_empty());

        routes
            .lock()
            .unwrap()
            .set(VoiceRouteId::Input, VoiceOutputTarget::MidiPort { port: 9 });
        send_input_passthrough(&[0xf8], &routes, &mut output);
        let trace = output.take_trace();
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].device_port, 9);
        assert_eq!(trace[0].message, [0xf8]);
    }

    #[test]
    fn generated_parts_resolve_to_distinct_stable_routes() {
        assert_eq!(main_voice_route(0, 3), VoiceRouteId::Input);
        assert_eq!(main_voice_route(1, 0), VoiceRouteId::Harmony { slot: 0 });
        assert_eq!(
            companion_voice_route("canon", 2),
            VoiceRouteId::Canon { voice: 2 }
        );
        assert_eq!(
            companion_voice_route("counterpoint", 1),
            VoiceRouteId::Counterpoint { voice: 1 }
        );
        assert_eq!(
            companion_voice_route("pattern_low", 0),
            VoiceRouteId::PatternLow
        );
        assert_eq!(
            companion_voice_route("pattern_counter", 0),
            VoiceRouteId::PatternCounter
        );
    }

    /// Issue #14 detection: external ports selected + all voices Synth →
    /// the warning must fire. This is the support-thread case ("MIDI out
    /// not producing messages for some users").
    #[test]
    fn test_detect_no_external_output_warning_fires_when_all_synth() {
        let voices = VoiceOutputRoutes::default();
        let result = detect_no_external_output_warning(&voices, &[0, 1]);
        assert!(
            result.is_some(),
            "expected a warning when no voice routes to an external port"
        );
        let msg = result.unwrap();
        assert!(
            msg.contains("external MIDI port"),
            "msg should mention external MIDI: {}",
            msg
        );
        assert!(
            msg.contains("internal synth"),
            "msg should explain where it's going: {}",
            msg
        );
    }

    /// At least one voice routed to an external MIDI port is the
    /// happy path — no warning even if other voices stay on Synth.
    #[test]
    fn test_detect_no_external_output_warning_silent_when_any_external() {
        let mut voices = VoiceOutputRoutes::default();
        voices.set(
            VoiceRouteId::Canon { voice: 2 },
            VoiceOutputTarget::MidiPort { port: 0 },
        );
        assert!(detect_no_external_output_warning(&voices, &[0]).is_none());
    }

    /// User chose synth-only (no external ports selected) → never warn.
    /// They explicitly opted out of MIDI; the bug doesn't apply.
    #[test]
    fn test_detect_no_external_output_warning_silent_when_no_external_ports_selected() {
        let voices = VoiceOutputRoutes::default();
        assert!(detect_no_external_output_warning(&voices, &[]).is_none());
    }

    /// All voices set to `Off` is a legitimate user choice (mute) — the
    /// warning logic only cares about MidiPort presence, so Off-only
    /// still warns since external ports were selected but nothing
    /// reaches them. That matches user intent: "I wanted MIDI out".
    #[test]
    fn test_detect_no_external_output_warning_fires_when_all_off() {
        let mut voices = VoiceOutputRoutes::default();
        voices.set(VoiceRouteId::Input, VoiceOutputTarget::Off);
        assert!(detect_no_external_output_warning(&voices, &[0]).is_some());
    }

    /// No frequency at all (idle / silence) must produce an empty name.
    #[test]
    fn test_build_guitar_signal_payload_no_frequency() {
        let silent = GuitarSignalInfo {
            rms: 0.001,
            frequency: None,
            clarity: 0.0,
            note_state: 0,
        };
        let payload = build_guitar_signal_payload(silent);
        assert_eq!(payload.note_name, "");
        assert_eq!(payload.midi_note, 0);
    }
}

#[cfg(test)]
#[path = "engine_determinism_tests.rs"]
mod deterministic_performance_tests;
