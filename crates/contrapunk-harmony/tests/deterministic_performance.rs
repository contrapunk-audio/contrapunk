use contrapunk_harmony::{HarmonyEngine, HarmonyMode, Key};
use wmidi::Note;

fn note(value: u8) -> Note {
    Note::try_from(value).expect("valid MIDI note")
}

fn semantic_trace(mode: HarmonyMode) -> Vec<(bool, u8, Vec<u8>)> {
    let mut engine = HarmonyEngine::with_voices(Key::C, mode, 4);
    engine.set_counterpoint_beat_phase(Some(0.0));
    let phrase = [60, 64, 62, 67, 65, 69, 71, 67, 72, 69, 65, 62];
    let mut trace = Vec::new();

    for (index, midi) in phrase.into_iter().enumerate() {
        engine.set_counterpoint_beat_phase(Some((index % 4) as f64));
        trace.push((
            true,
            midi,
            engine
                .harmonize_note_on(note(midi))
                .into_iter()
                .map(u8::from)
                .collect(),
        ));
    }
    for midi in phrase.into_iter().rev() {
        trace.push((
            false,
            midi,
            engine
                .harmonize_note_off(note(midi))
                .into_iter()
                .map(u8::from)
                .collect(),
        ));
    }
    trace
}

#[test]
fn every_selectable_harmony_mode_replays_identically() {
    for mode in HarmonyMode::all().iter().copied() {
        let expected = semantic_trace(mode);
        for run in 1..32 {
            assert_eq!(
                semantic_trace(mode),
                expected,
                "{} diverged on replay {run}",
                mode.description()
            );
        }
    }
}

#[test]
fn random_modes_are_not_selectable() {
    let selectable: Vec<_> = HarmonyMode::all()
        .iter()
        .map(HarmonyMode::description)
        .collect();
    assert!(
        selectable
            .iter()
            .all(|description| !description.to_ascii_lowercase().contains("random")),
        "ambient randomness is not a performance-safe harmony mode: {selectable:?}"
    );
}

#[test]
fn legacy_random_modes_migrate_to_deterministic_modes() {
    let random_below: HarmonyMode = serde_json::from_str("\"random_below\"").unwrap();
    let consonant: HarmonyMode = serde_json::from_str("\"random_below_no_seconds\"").unwrap();

    assert_eq!(random_below, HarmonyMode::ContraryMotion);
    assert_eq!(consonant, HarmonyMode::StrictCounterpoint);
}

#[test]
fn repeated_same_pitch_releases_each_generated_frame_in_attack_order() {
    let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::ContraryMotion, 4);
    let first = engine.harmonize_note_on(note(60));
    let second = engine.harmonize_note_on(note(60));

    assert_eq!(engine.harmonize_note_off(note(60)), first);
    assert_eq!(engine.harmonize_note_off(note(60)), second);
}

#[test]
fn repeated_held_pitch_keeps_each_owner_during_configuration_replay() {
    let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
    engine.harmonize_note_on(note(60));
    engine.harmonize_note_on(note(60));

    engine.set_key(Key::G);
    assert_eq!(engine.take_reharm_inputs(), vec![60, 60]);
}

#[test]
fn configuration_replay_preserves_same_pitch_source_owners() {
    let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
    engine.harmonize_note_on_owned(note(60), 3);
    engine.harmonize_note_on_owned(note(60), 1);

    engine.set_key(Key::G);
    assert_eq!(engine.take_owned_reharm_inputs(), vec![(1, 60), (3, 60)]);
}

#[test]
fn configuration_replay_preserves_source_velocity() {
    let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::ContraryMotion, 3);
    engine.harmonize_note_on_owned_with_velocity(note(60), 3, 41);
    engine.harmonize_note_on_owned_with_velocity(note(60), 1, 99);

    engine.set_mode(HarmonyMode::StrictCounterpoint);

    assert_eq!(
        engine.take_owned_reharm_inputs_with_velocity(),
        vec![(1, 60, 99), (3, 60, 41)]
    );
}

#[test]
fn pass_through_attack_can_gain_harmony_during_configuration_replay() {
    let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::PassThrough, 3);
    engine.harmonize_note_on_owned_with_velocity(note(60), 2, 73);

    engine.set_mode(HarmonyMode::DiatonicThirds);

    assert_eq!(
        engine.take_owned_reharm_inputs_with_velocity(),
        vec![(2, 60, 73)]
    );
}

#[test]
fn held_inputs_replay_in_canonical_pitch_order_after_configuration_change() {
    let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
    let held = [72, 48, 67, 55, 76, 60, 64, 52, 71, 57, 69, 50];
    for midi in held {
        engine.harmonize_note_on(note(midi));
    }

    engine.set_key(Key::G);
    let replay = engine.take_reharm_inputs();
    let mut sorted = replay.clone();
    sorted.sort_unstable();

    assert_eq!(
        replay, sorted,
        "reharmonization order must not depend on HashMap iteration"
    );
}
