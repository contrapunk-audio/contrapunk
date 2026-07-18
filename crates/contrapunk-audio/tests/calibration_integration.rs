//! Integration tests for the calibration round-trip.
//!
//! These exercise the seam between the calibration data model
//! (`GuitarCalibrationProfile`), filesystem persistence (the on-disk
//! JSON the Tauri layer also reads), and the `GuitarInput` pipeline
//! consumer (`set_calibration_profile`).
//!
//! The Tauri side has its own command-handler tests under
//! `src-tauri/src/commands/*`. These tests cover the audio-crate side
//! of the same round-trip so a regression here triggers separately
//! from a Tauri-only regression.

use contrapunk_audio::guitar::{CalibrationSample, GuitarCalibrationProfile};
use contrapunk_audio::guitar_input::GuitarInput;

fn realistic_profile() -> GuitarCalibrationProfile {
    let mut p = GuitarCalibrationProfile::default();
    let notes = ["E2", "A2", "D3", "G3", "B3", "E4"];
    for (i, s) in p.strings.iter_mut().enumerate() {
        for k in 0..4 {
            s.soft_samples.push(CalibrationSample {
                note: notes[i].to_string(),
                freq: 80.0 + i as f32 * 30.0,
                conf: 0.85 + k as f32 * 0.02,
                peak: 0.08,
                rms: 0.05 + k as f32 * 0.005,
                bright_peak: 0.04,
                bright_rms: 0.025,
                main_delta: 0.0,
                main_ratio: 0.0,
                main_slope: 0.0,
                bright_delta: 0.0,
                bright_ratio: 0.0,
                bright_slope: 0.0,
            });
            s.strong_samples.push(CalibrationSample {
                note: notes[i].to_string(),
                freq: 80.0 + i as f32 * 30.0,
                conf: 0.92 + k as f32 * 0.01,
                peak: 0.3,
                rms: 0.2 + k as f32 * 0.01,
                bright_peak: 0.15,
                bright_rms: 0.1,
                main_delta: 0.0,
                main_ratio: 0.0,
                main_slope: 0.0,
                bright_delta: 0.0,
                bright_ratio: 0.0,
                bright_slope: 0.0,
            });
        }
    }
    p
}

#[test]
fn profile_json_round_trip_with_samples_preserves_per_string_data() {
    let original = realistic_profile();
    let serialized = original.to_json().expect("serialize");
    let loaded = GuitarCalibrationProfile::from_json(&serialized).expect("parse");

    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.strings.len(), 6);
    for (i, (lhs, rhs)) in loaded
        .strings
        .iter()
        .zip(original.strings.iter())
        .enumerate()
    {
        assert_eq!(
            lhs.soft_samples.len(),
            rhs.soft_samples.len(),
            "string {} soft sample count mismatch",
            i
        );
        assert_eq!(
            lhs.strong_samples.len(),
            rhs.strong_samples.len(),
            "string {} strong sample count mismatch",
            i
        );
        for (a, b) in lhs.soft_samples.iter().zip(rhs.soft_samples.iter()) {
            assert!((a.rms - b.rms).abs() < 1e-6);
            assert!((a.conf - b.conf).abs() < 1e-6);
        }
    }
}

#[test]
fn filesystem_round_trip_via_temp_dir_picks_up_profile() {
    // Simulate the production flow: save JSON → read it back → apply
    // to the engine. Mirrors what the Tauri commands do at runtime.
    let path = unique_tmp_path("filesystem_round_trip.json");
    let original = realistic_profile();
    let serialized = original.to_json().expect("serialize");
    std::fs::write(&path, serialized.as_bytes()).expect("write");

    let raw = std::fs::read_to_string(&path).expect("read");
    let loaded = GuitarCalibrationProfile::from_json(&raw).expect("parse");

    let mut pipe = GuitarInput::with_defaults();
    assert!(!pipe.has_calibration_profile());
    pipe.set_calibration_profile(loaded);
    assert!(pipe.has_calibration_profile(), "profile must apply");

    // Profile attachment is observable via the stored profile.
    let stored = pipe
        .calibration_profile()
        .expect("profile present after set");
    assert_eq!(stored.strings.len(), 6);
    assert_eq!(stored.strings[0].soft_samples.len(), 4);
    // Lock the actual sample value, not just the count — guarantees the
    // round-trip didn't silently drop data (a previous version of this
    // test only checked length).
    let original_first = &realistic_profile().strings[0].soft_samples[0];
    let stored_first = &stored.strings[0].soft_samples[0];
    assert!((stored_first.rms - original_first.rms).abs() < 1e-6);
    assert!((stored_first.conf - original_first.conf).abs() < 1e-6);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn corrupt_json_is_rejected_with_a_serde_error() {
    // Verify the on-disk parser surfaces clear errors. The Tauri layer
    // bubbles these to the UI as `calibrationError`. We assert the
    // error CONTENT references serde's expected structural hints
    // ("key must be a string", "expected value", "EOF while parsing",
    // etc) — a bare is_err() check would pass even for a hypothetical
    // future non-parse error branch.
    let result = GuitarCalibrationProfile::from_json("{ this is not valid json");
    let err = result.expect_err("must error on truncated JSON");
    let msg = err.to_string().to_lowercase();
    let parse_hint = msg.contains("expected")
        || msg.contains("invalid")
        || msg.contains("syntax")
        || msg.contains("key must")
        || msg.contains("eof")
        || msg.contains("value");
    assert!(parse_hint, "expected a parse error, got: {msg}");
    // Also lock that the error carries position info (serde always
    // includes "line N column M"). If a future error variant drops
    // location, the UI will get an opaque message and this test trips.
    assert!(
        msg.contains("line") && msg.contains("column"),
        "serde error must surface line/column for UI display, got: {msg}"
    );
}

#[test]
fn empty_json_object_is_rejected_until_strings_field_becomes_default() {
    // The minimal object `{}` does not satisfy serde's required-field
    // contract for `GuitarCalibrationProfile` (no `version`, no
    // `strings`). This test locks that behavior — if a future schema
    // change adds `#[serde(default)]` so `{}` parses as the default
    // profile, this test trips intentionally and the schema-evolution
    // author has to decide: was the silent-default behavior intended,
    // or do we still want strict rejection at the storage boundary?
    let result = GuitarCalibrationProfile::from_json("{}");
    let err = result.expect_err("empty object must be rejected today");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("missing field") || msg.contains("expected"),
        "expected a missing-field error, got: {msg}"
    );
}

// NOTE: A previous integration test "post-reload-silence-is-clean"
// was deleted because it could not be expressed reliably through the
// public API. The pipeline's ring buffer holds a tail of the prior
// stimulus, so `process_block(&silence)` after a real-note stimulus
// can still emit a NoteOff from the buffered tail — the detector is
// doing the right thing; the test was asserting the wrong invariant.
// The active-note preservation contract is locked at the unit-test layer in
// `guitar_input.rs::set_calibration_profile_preserves_active_note_for_noteoff`,
// which reaches into private fields directly. Don't reintroduce a
// duplicative integration version without first solving the ring-
// buffer-tail problem (e.g. by exposing a `flush_ring_buffer()` API
// or by checking field state via a public introspection method).

#[test]
fn process_block_with_nan_does_not_crash_pipeline() {
    // Real audio interfaces occasionally feed NaN frames on driver
    // hiccups. The calibration consumer's `compute_bright_rms` does a
    // sqrt and a division; NaN must NOT escape the pipeline as a
    // panic or as a noise-rejection-of-everything state (the latter
    // would lock guitar input out for the rest of the session).
    let mut pipe = GuitarInput::with_defaults();
    pipe.set_calibration_profile(realistic_profile());

    let bs = pipe.config().buffer_size;
    let mut buf = vec![0.0f32; bs];
    // Inject a few NaN samples scattered through the buffer.
    buf[0] = f32::NAN;
    buf[bs / 2] = f32::NAN;
    buf[bs - 1] = f32::NAN;

    // Must not panic.
    let _events = pipe.process_block(&buf);

    // Subsequent CLEAN input must still be processable — a NaN
    // batch must not strand the pipeline in a state that blocks
    // future plucks.
    let clean = vec![0.0f32; bs];
    let _events2 = pipe.process_block(&clean);
    // Profile still attached after NaN exposure.
    assert!(pipe.has_calibration_profile());
}

#[test]
fn partially_calibrated_profile_does_not_crash_set() {
    // Production users may run a partial calibration sweep (e.g. only
    // 3 of 6 strings have samples). The previous tests use a fully-
    // populated profile; this one exercises the
    // `from_profile` empty-branch path at guitar.rs:614-618.
    let mut p = GuitarCalibrationProfile::default();
    // Only populate the low E string.
    p.strings[0].soft_samples.push(CalibrationSample {
        note: "E2".to_string(),
        freq: 82.41,
        conf: 0.9,
        peak: 0.08,
        rms: 0.05,
        bright_peak: 0.04,
        bright_rms: 0.025,
        main_delta: 0.0,
        main_ratio: 0.0,
        main_slope: 0.0,
        bright_delta: 0.0,
        bright_ratio: 0.0,
        bright_slope: 0.0,
    });
    p.strings[0].strong_samples.push(CalibrationSample {
        note: "E2".to_string(),
        freq: 82.41,
        conf: 0.95,
        peak: 0.3,
        rms: 0.2,
        bright_peak: 0.15,
        bright_rms: 0.1,
        main_delta: 0.0,
        main_ratio: 0.0,
        main_slope: 0.0,
        bright_delta: 0.0,
        bright_ratio: 0.0,
        bright_slope: 0.0,
    });
    // Strings 1..5 left empty.

    let mut pipe = GuitarInput::with_defaults();
    pipe.set_calibration_profile(p);
    assert!(pipe.has_calibration_profile());

    // Processing silence with a partial profile must not panic.
    let bs = pipe.config().buffer_size;
    let silence = vec![0.0f32; bs * 2];
    let events = pipe.process_block(&silence);
    assert!(events.is_empty());
}

/// Generate a unique-to-this-process tmp file path. Avoids cross-run
/// collisions when `cargo test` runs the integration tests in
/// parallel — the previous (fixed-path) helper was a race waiting to
/// happen.
fn unique_tmp_path(stem: &str) -> std::path::PathBuf {
    let base = std::env::temp_dir().join("contrapunk_audio_calibration_tests");
    std::fs::create_dir_all(&base).expect("create test tmpdir");
    let nonce = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    base.join(format!("{stem}.{nonce}.{nanos}"))
}
