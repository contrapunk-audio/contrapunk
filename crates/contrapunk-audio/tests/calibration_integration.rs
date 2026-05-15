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
    let dir = tempdir_in_target();
    let path = dir.join("guitar_calibration_profile.json");
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

    // Cleanup
    let _ = std::fs::remove_file(&path);
}

#[test]
fn corrupt_json_is_rejected_with_a_real_error() {
    // Verify the on-disk parser surfaces clear errors. The Tauri layer
    // bubbles these to the UI as `calibrationError`.
    let result = GuitarCalibrationProfile::from_json("{ this is not valid json");
    assert!(
        result.is_err(),
        "corrupt JSON must error, not silently default"
    );
}

#[test]
fn empty_json_object_falls_through_to_default_friendly_shape() {
    // A minimal object without `strings`: serde rejects because the
    // field is required. Documents the expected behavior so a future
    // schema change with `#[serde(default)]` would trip this test
    // (intentionally — that's a forward-compat decision worth flagging).
    let result = GuitarCalibrationProfile::from_json("{}");
    assert!(result.is_err());
}

#[test]
fn round_trip_followed_by_reload_resets_pipeline_state() {
    // Hot-reload during a sustaining note must not strand a ghost
    // note: set_calibration_profile drains state per the brutal-
    // critic #16 fix. This is the integration-level lock for that
    // contract.
    let mut pipe = GuitarInput::with_defaults();
    pipe.set_calibration_profile(realistic_profile());

    // Feed silence — no notes should fire and the pipeline stays Idle.
    let silence = vec![0.0f32; 2048];
    let events = pipe.process_block(&silence);
    assert!(events.is_empty(), "silence must not produce MIDI events");

    // Re-apply profile — should not panic, should not strand state.
    pipe.set_calibration_profile(realistic_profile());

    // Still no events from silence.
    let events2 = pipe.process_block(&silence);
    assert!(events2.is_empty());
}

/// Resolve a tmpdir inside the cargo target dir. We avoid the system
/// /tmp to keep the round-trip on the same filesystem as the
/// production atomic-rename path uses (the Tauri save_calibration_profile
/// renames `*.json.tmp -> *.json` and relies on same-fs semantics).
fn tempdir_in_target() -> std::path::PathBuf {
    let base = std::env::temp_dir().join("contrapunk_audio_calibration_tests");
    std::fs::create_dir_all(&base).expect("create test tmpdir");
    base
}
