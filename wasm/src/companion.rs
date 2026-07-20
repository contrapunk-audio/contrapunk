//! WASM bridge for the shared Companion orchestrator.
//!
//! Exposes a thin `CompanionWasm` wrapper around
//! `contrapunk_companion::Companion` so the browser build can host
//! both lanes (Canon + Counterpoint) that ship in v1.2.0. The JS
//! adapter feeds player NoteOn / NoteOff via `on_note_on` /
//! `on_note_off`, ticks the lanes via `tick(beat)`, and reads back
//! dispatch ops as JSON to schedule on the WebAudio synth.
//!
//! The Companion's internal mini-engines need a HarmonyEngine
//! snapshot to mirror global key/scale/mode state. In Tauri that
//! snapshot lives in `WorldState::engine_snapshot` and is shared with
//! the router thread's main engine. In WASM there's no router thread;
//! the JS side calls `set_global_state(...)` after each user change
//! to mirror the same fields into the snapshot.

use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wmidi::Note;

use contrapunk_companion::lane::InputEvent;
use contrapunk_companion::{CanonLane, CounterpointLane, PatternLane};
use contrapunk_companion::{Companion, DispatchOp, WorldState};
use contrapunk_harmony::{HarmonyEngine, HarmonyMode, Key};
use contrapunk_transport::Transport;

use crate::enum_strings::{parse_key, parse_mode, parse_scale_mode};

#[wasm_bindgen]
pub struct CompanionWasm {
    inner: Companion,
    world: Arc<WorldState>,
    /// Unused "main engine" passed into Companion::tick — the canon
    /// and counterpoint lanes are both Decide-phase and don't mutate
    /// the main engine, so this is a no-op recipient.
    dummy_engine: Mutex<HarmonyEngine>,
}

#[wasm_bindgen]
impl CompanionWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let transport = Transport::new(48_000);
        // Snapshot engine the Companion's mini-engines read for global
        // key/mode/scale defaults. JS keeps this in sync via
        // set_global_* setters below.
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::PassThrough,
        )));
        let world = WorldState::new(Arc::clone(&transport), Arc::clone(&engine));
        let mut companion = Companion::new(world.clone());
        companion.lanes.push(Box::new(CanonLane::new()));
        companion.lanes.push(Box::new(CounterpointLane::new()));
        companion
            .lanes
            .push(Box::new(PatternLane::new("Low Support", "pattern_low")));
        companion.lanes.push(Box::new(PatternLane::new(
            "Counterline Pattern",
            "pattern_counter",
        )));
        // Master enable defaults ON to match the Tauri build's FTUX.
        companion
            .enabled
            .store(true, std::sync::atomic::Ordering::Release);

        Self {
            inner: companion,
            world,
            dummy_engine: Mutex::new(HarmonyEngine::new(Key::C, HarmonyMode::PassThrough)),
        }
    }

    #[wasm_bindgen]
    pub fn set_enabled(&self, enabled: bool) {
        self.inner
            .enabled
            .store(enabled, std::sync::atomic::Ordering::Release);
    }

    #[wasm_bindgen]
    pub fn is_enabled(&self) -> bool {
        self.inner
            .enabled
            .load(std::sync::atomic::Ordering::Acquire)
    }

    /// Mirror the global engine's key/scale/mode/voice_count etc.
    /// into the snapshot the Companion's mini-engines read. Called by
    /// JS whenever the Harmony tab changes these.
    #[wasm_bindgen]
    pub fn set_global_state(
        &self,
        key: &str,
        mode: &str,
        scale_mode: &str,
        voice_count: usize,
        voice_position: usize,
    ) -> Result<(), JsValue> {
        let key = parse_key(key)?;
        let mode = parse_mode(mode)?;
        let scale_mode = parse_scale_mode(scale_mode)?;
        let mut eng = self
            .world
            .engine_snapshot
            .lock()
            .map_err(|e| JsValue::from_str(&format!("snapshot lock: {}", e)))?;
        if eng.key() != key {
            eng.set_key(key);
        }
        if eng.mode() != mode {
            eng.set_mode(mode);
        }
        if eng.scale_mode() != scale_mode {
            eng.set_scale_mode(scale_mode);
        }
        if eng.voice_count() != voice_count {
            eng.set_voice_count(voice_count);
        }
        if eng.voice_position() != voice_position {
            eng.set_voice_position(voice_position);
        }
        Ok(())
    }

    /// Apply a partial JSON state blob to the canon lane (same shape
    /// the Tauri command consumes). See CanonLane::deserialize_state.
    #[wasm_bindgen]
    pub fn configure_canon(&mut self, json: &str) -> Result<(), JsValue> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner
            .configure_lane("canon", value)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn configure_counterpoint(&mut self, json: &str) -> Result<(), JsValue> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner
            .configure_lane("counterpoint", value)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn configure_pattern(&mut self, lane_id: &str, json: &str) -> Result<(), JsValue> {
        if !matches!(lane_id, "pattern_low" | "pattern_counter") {
            return Err(JsValue::from_str("unknown pattern lane"));
        }
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner
            .configure_lane(lane_id, value)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Set the Companion's global HoldMode default. JSON shape:
    ///   {"kind":"cancel"}
    ///   {"kind":"near_future","tail_beats":1.0}
    ///   {"kind":"phrase_end"}
    ///   {"kind":"forever"}
    /// Lanes / voices can still override via the existing
    /// `configure_canon` / `configure_counterpoint` JSON paths (they
    /// each accept a `hold_mode` field with the same shape).
    #[wasm_bindgen]
    pub fn pattern_state(&self, lane_id: &str) -> Result<String, JsValue> {
        if !matches!(lane_id, "pattern_low" | "pattern_counter") {
            return Err(JsValue::from_str("unknown pattern lane"));
        }
        Ok(self
            .inner
            .lane_state(lane_id)
            .unwrap_or(serde_json::Value::Null)
            .to_string())
    }

    #[wasm_bindgen]
    pub fn set_global_hold_mode(&self, json: &str) -> Result<(), JsValue> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mode = contrapunk_companion::lane::hold_mode_from_json(&value)
            .ok_or_else(|| JsValue::from_str("invalid hold_mode JSON shape"))?;
        self.inner.set_global_hold_mode(mode);
        Ok(())
    }

    /// Advance the transport by `frames` audio frames. JS calls this
    /// from its animation-frame / WebAudio loop so per-lane scheduling
    /// (canon delays, counterpoint subdivisions) sees forward time.
    /// At the default 48 kHz sample rate, 768 frames ≈ 16 ms.
    #[wasm_bindgen]
    pub fn advance(&self, frames: u32) {
        if !self.world.transport.is_running() {
            self.world.transport.play();
        }
        let _ = self.world.transport.advance(frames);
    }

    /// Feed a player NoteOn into the Companion. Returns a JSON array
    /// of dispatch ops emitted by lanes that fired immediately
    /// (Species 1 canon-onset emissions, etc.). Each op carries a
    /// `lane` field (e.g. `"canon"`, `"counterpoint"`) so the UI can
    /// render per-lane attribution (different piano colors per lane).
    #[wasm_bindgen]
    pub fn on_note_on(&mut self, note: u8, velocity: u8, channel: u8) -> Result<String, JsValue> {
        let ev = InputEvent::NoteOn {
            note,
            velocity,
            channel,
        };
        let (tagged, _suppress) = self.inner.on_input_tagged(ev, &self.dummy_engine);
        serialize_tagged_ops(&tagged).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn on_note_off(&mut self, note: u8, channel: u8) -> Result<String, JsValue> {
        let ev = InputEvent::NoteOff { note, channel };
        let (tagged, _suppress) = self.inner.on_input_tagged(ev, &self.dummy_engine);
        serialize_tagged_ops(&tagged).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Tick the lanes. Drains pending emissions whose fire_at has
    /// elapsed. Returns a JSON array of dispatch ops to schedule on
    /// the WebAudio synth. Each op carries its originating `lane`.
    #[wasm_bindgen]
    pub fn tick(&mut self) -> Result<String, JsValue> {
        let tagged = self.inner.tick_tagged(&self.dummy_engine);
        serialize_tagged_ops(&tagged).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Clear delayed/held lane state without changing configuration.
    #[wasm_bindgen]
    pub fn reset_runtime(&mut self) {
        self.inner.reset_runtime();
    }

    /// Debug snapshot — JSON dump of the global engine snapshot + each
    /// canon-lane voice's mini-engine state (mode, key, scale_mode,
    /// voice_count, voice_position) and the voice config (transpose,
    /// reference_voice, etc.). Used by the JS adapter to log what's
    /// actually happening on each NoteOn so we can verify the per-voice
    /// engines are independent and the cascade is firing.
    #[wasm_bindgen]
    pub fn debug_snapshot(&self) -> String {
        let snapshot = if let Ok(g) = self.world.engine_snapshot.lock() {
            serde_json::json!({
                "key": format!("{:?}", g.key()),
                "mode": format!("{:?}", g.mode()),
                "scale_mode": format!("{:?}", g.scale_mode()),
                "voice_count": g.voice_count(),
                "voice_position": g.voice_position(),
                "counterpoint_species": format!("{:?}", g.counterpoint_species()),
            })
        } else {
            serde_json::json!({ "error": "snapshot lock poisoned" })
        };
        // Canon lane state — already serialized by CanonLane itself
        // (includes per-voice config). We add the global snapshot
        // alongside so a single log line shows both.
        let canon = self
            .inner
            .lane_state("canon")
            .unwrap_or(serde_json::Value::Null);
        let counterpoint = self
            .inner
            .lane_state("counterpoint")
            .unwrap_or(serde_json::Value::Null);
        let pattern_low = self
            .inner
            .lane_state("pattern_low")
            .unwrap_or(serde_json::Value::Null);
        let pattern_counter = self
            .inner
            .lane_state("pattern_counter")
            .unwrap_or(serde_json::Value::Null);
        serde_json::json!({
            "snapshot": snapshot,
            "canon": canon,
            "counterpoint": counterpoint,
            "pattern_low": pattern_low,
            "pattern_counter": pattern_counter,
            "companion_enabled": self.inner.enabled.load(std::sync::atomic::Ordering::Acquire),
            "transport_beats": self.world.transport.total_beats(),
        })
        .to_string()
    }
}

impl Default for CompanionWasm {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialize dispatch ops in a stable shape for the JS adapter.
/// Mirrors the JSON the Tauri command path produces so the UI side
/// can share decoder code.
#[allow(dead_code)]
fn serialize_ops(ops: &[DispatchOp]) -> Result<String, serde_json::Error> {
    let values: Vec<serde_json::Value> = ops.iter().map(op_to_json).collect();
    serde_json::to_string(&values)
}

/// Same shape as `serialize_ops` but each entry carries an extra
/// `lane` field — the `type_id` of the lane that produced it
/// (e.g. `"canon"` or `"counterpoint"`). Used by the JS adapter to
/// attribute each emission to a lane so the UI can color piano keys
/// per-lane.
fn serialize_tagged_ops(
    tagged: &[(&'static str, DispatchOp)],
) -> Result<String, serde_json::Error> {
    let values: Vec<serde_json::Value> = tagged
        .iter()
        .map(|(lane_id, op)| {
            let mut v = op_to_json(op);
            if let serde_json::Value::Object(ref mut map) = v {
                // serde_json::Value::String requires owned — one
                // alloc here at JSON-encode time is unavoidable.
                // Everything else in the dispatch path is alloc-free.
                map.insert(
                    "lane".into(),
                    serde_json::Value::String((*lane_id).to_string()),
                );
            }
            v
        })
        .collect();
    serde_json::to_string(&values)
}

fn op_to_json(op: &DispatchOp) -> serde_json::Value {
    match op {
        DispatchOp::NoteOn {
            note,
            velocity,
            channel,
            ..
        } => serde_json::json!({
            "kind": "note_on",
            "note": note,
            "velocity": velocity,
            "channel": channel,
        }),
        DispatchOp::NoteOff { note, channel, .. } => serde_json::json!({
            "kind": "note_off",
            "note": note,
            "channel": channel,
        }),
        DispatchOp::AllNotesOff { ports } => serde_json::json!({
            "kind": "all_notes_off",
            "ports": ports,
        }),
    }
}

// Silence unused-import warnings on Note when wasm-bindgen drops in
// generated glue that doesn't reference it.
#[allow(dead_code)]
fn _note_typeguard(_n: Note) {}
